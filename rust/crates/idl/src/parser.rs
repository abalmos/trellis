use crate::{
    ast::{
        Api, ApiUse, Binding, Capability, Constraint, ConstraintValue, Docs, ErrorDecl, Field,
        Participant, Project, Resource, SchemaDecl, Selection, Source, Spanned, State, Surface,
        Transfer, Type,
    },
    lexer::{lex, Token, TokenKind},
};
use miette::{LabeledSpan, MietteDiagnostic, NamedSource, Report};
use std::collections::{btree_map::Entry, BTreeMap};

pub(crate) fn parse(sources: Vec<Source>) -> miette::Result<Project> {
    let mut project = Project {
        sources,
        apis: Vec::new(),
        participants: Vec::new(),
    };
    for source_index in 0..project.sources.len() {
        let source = &project.sources[source_index];
        let tokens = lex(&source.text)
            .map_err(|span| diagnostic(source, span, "unrecognized token in Trellis IDL"))?;
        let mut parser = Parser {
            source,
            source_index,
            tokens,
            position: 0,
        };
        while !parser.done() {
            if parser.at_word("api") {
                project.apis.push(parser.api()?);
            } else if parser.at_word("participant") {
                project.participants.push(parser.participant()?);
            } else {
                return Err(parser.error_here("expected 'api' or 'participant'"));
            }
        }
    }
    Ok(project)
}

struct Parser<'a> {
    source: &'a Source,
    source_index: usize,
    tokens: Vec<Token>,
    position: usize,
}

impl Parser<'_> {
    fn api(&mut self) -> miette::Result<Spanned<Api>> {
        let start = self.word("api")?.start;
        let id = self.string()?.value;
        self.token(TokenKind::LBrace)?;
        let mut api = Api {
            id,
            version: None,
            display_name: None,
            description: None,
            docs: None,
            schemas: BTreeMap::new(),
            exports: Vec::new(),
            capabilities: BTreeMap::new(),
            errors: BTreeMap::new(),
            rpcs: BTreeMap::new(),
            operations: BTreeMap::new(),
            events: BTreeMap::new(),
            feeds: BTreeMap::new(),
        };
        while !self.at(TokenKind::RBrace) {
            match self.word_text()?.as_str() {
                "version" => api.version = Some(self.string_statement()?),
                "display_name" => api.display_name = Some(self.string_statement()?),
                "description" => api.description = Some(self.string_statement()?),
                "docs" => api.docs = Some(self.docs()?),
                "model" => {
                    let (name, declaration) = self.model()?;
                    insert(&mut api.schemas, name, declaration, self.source)?;
                }
                "type" => {
                    let (name, declaration) = self.alias()?;
                    insert(&mut api.schemas, name, declaration, self.source)?;
                }
                "enum" => {
                    let (name, declaration) = self.enum_decl()?;
                    insert(&mut api.schemas, name, declaration, self.source)?;
                }
                "export" => {
                    api.exports.push(self.ident_statement()?);
                }
                "capability" => {
                    let (name, capability) = self.capability()?;
                    insert(&mut api.capabilities, name, capability, self.source)?;
                }
                "error" => {
                    let name = self.ident()?;
                    let start = name.span.start;
                    let mut value = ErrorDecl::default();
                    let end = if self.eat(TokenKind::Semi) {
                        self.previous_span().end
                    } else {
                        self.token(TokenKind::LBrace)?;
                        while !self.at(TokenKind::RBrace) {
                            match self.word_text()?.as_str() {
                                "code" => value.code = Some(self.string_statement()?),
                                "schema" => value.schema = Some(self.ident_statement()?),
                                other => {
                                    return Err(self.error_previous(format!(
                                        "unsupported error member '{other}'"
                                    )))
                                }
                            }
                        }
                        self.token(TokenKind::RBrace)?.end
                    };
                    let declaration = self.spanned(value, start..end);
                    insert(&mut api.errors, name, declaration, self.source)?;
                }
                "rpc" => {
                    let (name, surface) = self.surface()?;
                    insert(&mut api.rpcs, name, surface, self.source)?;
                }
                "operation" => {
                    let (name, surface) = self.surface()?;
                    insert(&mut api.operations, name, surface, self.source)?;
                }
                "event" => {
                    let (name, surface) = self.surface()?;
                    insert(&mut api.events, name, surface, self.source)?;
                }
                "feed" => {
                    let (name, surface) = self.surface()?;
                    insert(&mut api.feeds, name, surface, self.source)?;
                }
                other => {
                    return Err(
                        self.error_previous(format!("unsupported API declaration '{other}'"))
                    )
                }
            }
        }
        let end = self.token(TokenKind::RBrace)?.end;
        Ok(self.spanned(api, start..end))
    }

    fn capability(&mut self) -> miette::Result<(Spanned<String>, Spanned<Capability>)> {
        let name = self.string()?;
        let start = name.span.start;
        if self.at(TokenKind::Semi) {
            let end = self.token(TokenKind::Semi)?.end;
            return Ok((name, self.spanned(Capability::default(), start..end)));
        }
        self.token(TokenKind::LBrace)?;
        let mut capability = Capability::default();
        while !self.at(TokenKind::RBrace) {
            match self.word_text()?.as_str() {
                "display_name" => capability.display_name = Some(self.string_statement()?),
                "description" => capability.description = Some(self.string_statement()?),
                "consequence" => capability.consequence = Some(self.string_statement()?),
                other => {
                    return Err(
                        self.error_previous(format!("unsupported capability member '{other}'"))
                    )
                }
            }
        }
        let end = self.token(TokenKind::RBrace)?.end;
        Ok((name, self.spanned(capability, start..end)))
    }

    fn model(&mut self) -> miette::Result<(Spanned<String>, Spanned<SchemaDecl>)> {
        let name = self.ident()?;
        let start = name.span.start;
        self.token(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) {
            let field = if self.at(TokenKind::String) {
                self.string()?
            } else {
                self.ident()?
            };
            let optional = self.eat(TokenKind::Question);
            self.token(TokenKind::Colon)?;
            let ty = self.ty()?;
            self.token(TokenKind::Semi)?;
            fields.push(Field {
                name: field.value,
                optional,
                ty,
            });
        }
        let end = self.token(TokenKind::RBrace)?.end;
        Ok((name, self.spanned(SchemaDecl::Model(fields), start..end)))
    }

    fn alias(&mut self) -> miette::Result<(Spanned<String>, Spanned<SchemaDecl>)> {
        let name = self.ident()?;
        let start = name.span.start;
        self.token(TokenKind::Eq)?;
        let ty = self.ty()?;
        let end = self.token(TokenKind::Semi)?.end;
        Ok((name, self.spanned(SchemaDecl::Alias(ty.value), start..end)))
    }

    fn enum_decl(&mut self) -> miette::Result<(Spanned<String>, Spanned<SchemaDecl>)> {
        let name = self.ident()?;
        let start = name.span.start;
        self.token(TokenKind::LBrace)?;
        let mut values = Vec::new();
        while !self.at(TokenKind::RBrace) {
            let value = if self.at(TokenKind::String) {
                self.string()?.value
            } else {
                self.ident()?.value
            };
            self.token(TokenKind::Semi)?;
            values.push(value);
        }
        let end = self.token(TokenKind::RBrace)?.end;
        Ok((name, self.spanned(SchemaDecl::Enum(values), start..end)))
    }

    fn ty(&mut self) -> miette::Result<Spanned<Type>> {
        let start = self.current_span().start;
        let mut members = vec![self.type_member()?];
        while self.eat(TokenKind::Pipe) {
            members.push(self.type_member()?);
        }
        let end = members.last().expect("type has a member").span.end;
        if members.len() == 1 {
            Ok(members.pop().expect("type has a member"))
        } else {
            Ok(self.spanned(Type::Union(members), start..end))
        }
    }

    fn type_member(&mut self) -> miette::Result<Spanned<Type>> {
        if self.at(TokenKind::String) {
            let value = self.string()?;
            return Ok(self.spanned(Type::Literal(value.value), value.span));
        }
        let name = self.ident()?;
        let start = name.span.start;
        let ty = match name.value.as_str() {
            "json" => Type::Json,
            "string" => Type::String(self.constraints()?),
            "bool" => Type::Bool,
            "true" => Type::BoolLiteral(true),
            "false" => Type::BoolLiteral(false),
            "uint" => Type::Integer {
                unsigned: true,
                constraints: self.constraints()?,
            },
            "int" => Type::Integer {
                unsigned: false,
                constraints: self.constraints()?,
            },
            "number" => Type::Number(self.constraints()?),
            "list" | "map" => {
                self.token(TokenKind::LAngle)?;
                let member = self.ty()?;
                self.token(TokenKind::RAngle)?;
                let constraints = self.constraints()?;
                let end = self.previous_span().end;
                return Ok(self.spanned(
                    if name.value == "list" {
                        Type::List {
                            member: Box::new(member),
                            constraints,
                        }
                    } else {
                        if !constraints.is_empty() {
                            return Err(self.error_at(&name, "map constraints are not supported"));
                        }
                        Type::Map(Box::new(member))
                    },
                    start..end,
                ));
            }
            "null" => Type::Null,
            _ => Type::Named(name.value),
        };
        let end = self.previous_span().end;
        Ok(self.spanned(ty, start..end))
    }

    fn constraints(&mut self) -> miette::Result<Vec<Constraint>> {
        if !self.eat(TokenKind::LParen) {
            return Ok(Vec::new());
        }
        let mut constraints = Vec::new();
        while !self.at(TokenKind::RParen) {
            let name = self.ident()?.value;
            self.token(TokenKind::Eq)?;
            let value = if self.at(TokenKind::String) {
                ConstraintValue::String(self.string()?.value)
            } else {
                let token = self.token(TokenKind::Number)?;
                ConstraintValue::Integer(self.text(&token).parse().map_err(|_| {
                    diagnostic(
                        self.source,
                        token.clone(),
                        "integer constraint is too large",
                    )
                })?)
            };
            constraints.push(Constraint { name, value });
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.token(TokenKind::RParen)?;
        Ok(constraints)
    }

    fn surface(&mut self) -> miette::Result<(Spanned<String>, Spanned<Surface>)> {
        let name = self.string()?;
        let start = name.span.start;
        self.token(TokenKind::LBrace)?;
        let mut surface = Surface::default();
        while !self.at(TokenKind::RBrace) {
            match self.word_text()?.as_str() {
                "version" => surface.version = Some(self.string_statement()?),
                "input" => surface.input = Some(self.ident_statement()?),
                "output" => surface.output = Some(self.ident_statement()?),
                "progress" => surface.progress = Some(self.ident_statement()?),
                "payload" | "event" => surface.event = Some(self.ident_statement()?),
                "params" => surface.params = self.string_list_statement()?,
                "errors" => surface.errors = self.ident_list_statement()?,
                "transfer" => {
                    let direction = self.ident()?;
                    surface.transfer = Some(match direction.value.as_str() {
                        "send" => Transfer::Send,
                        "receive" => Transfer::Receive,
                        _ => return Err(self.error_at(&direction, "expected 'send' or 'receive'")),
                    });
                    self.token(TokenKind::Semi)?;
                }
                "cancellable" => {
                    surface.cancellable = true;
                    self.token(TokenKind::Semi)?;
                }
                "capabilities" => surface.capabilities = self.capabilities()?,
                "subject" => surface.subject = Some(self.string_statement()?),
                "class" => surface.class = Some(self.ident_statement()?),
                "docs" => surface.docs = Some(self.docs()?),
                other => {
                    return Err(self.error_previous(format!("unsupported surface member '{other}'")))
                }
            }
        }
        let end = self.token(TokenKind::RBrace)?.end;
        Ok((name, self.spanned(surface, start..end)))
    }

    fn capabilities(&mut self) -> miette::Result<BTreeMap<String, Vec<String>>> {
        self.token(TokenKind::LBrace)?;
        let mut capabilities = BTreeMap::new();
        while !self.at(TokenKind::RBrace) {
            let action = self.ident()?.value;
            let names = self.string_list_statement()?;
            capabilities.insert(action, names);
        }
        self.token(TokenKind::RBrace)?;
        Ok(capabilities)
    }

    fn docs(&mut self) -> miette::Result<Docs> {
        self.token(TokenKind::LBrace)?;
        let mut docs = Docs::default();
        while !self.at(TokenKind::RBrace) {
            match self.word_text()?.as_str() {
                "summary" => docs.summary = Some(self.string_statement()?.value),
                "markdown" => docs.markdown = Some(self.string_statement()?.value),
                other => {
                    return Err(self.error_previous(format!("unsupported docs member '{other}'")))
                }
            }
        }
        self.token(TokenKind::RBrace)?;
        Ok(docs)
    }

    fn participant(&mut self) -> miette::Result<Spanned<Participant>> {
        let start = self.word("participant")?.start;
        let id = self.string()?.value;
        let kind = self.ident()?.value;
        self.token(TokenKind::LBrace)?;
        let mut participant = Participant {
            id,
            kind,
            implements: Vec::new(),
            uses: BTreeMap::new(),
            subscribed_events: Vec::new(),
            schemas: BTreeMap::new(),
            state: BTreeMap::new(),
            stores: BTreeMap::new(),
            kv: BTreeMap::new(),
            jobs: BTreeMap::new(),
            bindings: BTreeMap::new(),
        };
        while !self.at(TokenKind::RBrace) {
            match self.word_text()?.as_str() {
                "implements" => participant.implements.push(self.string_statement()?),
                "use" => {
                    let (alias, api_use) = self.api_use()?;
                    insert(&mut participant.uses, alias, api_use, self.source)?;
                }
                "subscribe" => {
                    self.word("event")?;
                    participant.subscribed_events.push(self.string_statement()?);
                }
                "model" => {
                    let (name, schema) = self.model()?;
                    insert(&mut participant.schemas, name, schema, self.source)?;
                }
                "type" => {
                    let (name, schema) = self.alias()?;
                    insert(&mut participant.schemas, name, schema, self.source)?;
                }
                "enum" => {
                    let (name, schema) = self.enum_decl()?;
                    insert(&mut participant.schemas, name, schema, self.source)?;
                }
                "state" => {
                    let name = self.ident()?;
                    let kind = self.ident()?.value;
                    let start = name.span.start;
                    self.token(TokenKind::LBrace)?;
                    let mut schema = None;
                    let mut state_version = None;
                    let mut docs = None;
                    while !self.at(TokenKind::RBrace) {
                        match self.word_text()?.as_str() {
                            "schema" => schema = Some(self.ident_statement()?),
                            "state_version" => state_version = Some(self.string_statement()?.value),
                            "docs" => docs = Some(self.docs()?),
                            other => {
                                return Err(self
                                    .error_previous(format!("unsupported state member '{other}'")))
                            }
                        }
                    }
                    let end = self.token(TokenKind::RBrace)?.end;
                    let schema =
                        schema.ok_or_else(|| self.error_here("state requires 'schema'"))?;
                    insert(
                        &mut participant.state,
                        name,
                        self.spanned(
                            State {
                                kind,
                                schema,
                                state_version,
                                docs,
                            },
                            start..end,
                        ),
                        self.source,
                    )?;
                }
                "store" => {
                    let (name, resource) = self.resource()?;
                    insert(&mut participant.stores, name, resource, self.source)?;
                }
                "kv" => {
                    let (name, resource) = self.resource()?;
                    insert(&mut participant.kv, name, resource, self.source)?;
                }
                "job" => {
                    let (name, resource) = self.resource()?;
                    insert(&mut participant.jobs, name, resource, self.source)?;
                }
                "bind" => {
                    self.word("operation")?;
                    let name = self.string()?;
                    let binding = self.binding()?;
                    insert(&mut participant.bindings, name, binding, self.source)?;
                }
                other => {
                    return Err(self
                        .error_previous(format!("unsupported participant declaration '{other}'")))
                }
            }
        }
        let end = self.token(TokenKind::RBrace)?.end;
        Ok(self.spanned(participant, start..end))
    }

    fn api_use(&mut self) -> miette::Result<(Spanned<String>, Spanned<ApiUse>)> {
        let requirement = self.ident()?;
        let required = match requirement.value.as_str() {
            "required" => true,
            "optional" => false,
            _ => return Err(self.error_at(&requirement, "expected 'required' or 'optional'")),
        };
        let alias = self.ident()?;
        let start = requirement.span.start;
        let api = self.string()?;
        self.token(TokenKind::LBrace)?;
        let mut selections = Vec::new();
        while !self.at(TokenKind::RBrace) {
            let action = self.ident()?;
            let surface = self.ident()?;
            if !matches!(
                (action.value.as_str(), surface.value.as_str()),
                ("call", "rpc")
                    | ("invoke" | "observe" | "cancel" | "control", "operation")
                    | ("publish" | "subscribe", "event")
                    | ("subscribe", "feed")
                    | ("read" | "write", "state")
            ) {
                return Err(self.error_at(
                    &action,
                    format!(
                        "unsupported participant use selection '{} {}'",
                        action.value, surface.value
                    ),
                ));
            }
            let name = self.string()?;
            let signal = if action.value == "control" {
                self.word("signal")?;
                Some(self.string()?.value)
            } else {
                None
            };
            let end = self.token(TokenKind::Semi)?.end;
            selections.push(self.spanned(
                Selection {
                    action: action.value,
                    surface: surface.value,
                    name: name.value,
                    signal,
                },
                action.span.start..end,
            ));
        }
        let end = self.token(TokenKind::RBrace)?.end;
        Ok((
            alias,
            self.spanned(
                ApiUse {
                    required,
                    api,
                    selections,
                },
                start..end,
            ),
        ))
    }

    fn resource(&mut self) -> miette::Result<(Spanned<String>, Spanned<Resource>)> {
        let name = self.ident()?;
        let start = name.span.start;
        self.token(TokenKind::LBrace)?;
        let mut resource = Resource::default();
        while !self.at(TokenKind::RBrace) {
            match self.word_text()?.as_str() {
                "purpose" => resource.purpose = Some(self.string_statement()?.value),
                "schema" => resource.schema = Some(self.ident_statement()?),
                "payload" => resource.payload = Some(self.ident_statement()?),
                "result" => resource.result = Some(self.ident_statement()?),
                "history" => resource.history = Some(self.number_statement()?),
                "ttl_ms" => resource.ttl_ms = Some(self.number_statement()?),
                "max_object_bytes" => resource.max_object_bytes = Some(self.number_statement()?),
                "max_total_bytes" => resource.max_total_bytes = Some(self.number_statement()?),
                "max_value_bytes" => resource.max_value_bytes = Some(self.number_statement()?),
                "docs" => resource.docs = Some(self.docs()?),
                other => {
                    return Err(
                        self.error_previous(format!("unsupported resource member '{other}'"))
                    )
                }
            }
        }
        let end = self.token(TokenKind::RBrace)?.end;
        Ok((name, self.spanned(resource, start..end)))
    }

    fn binding(&mut self) -> miette::Result<Spanned<Binding>> {
        let start = self.token(TokenKind::LBrace)?.start;
        self.word("transfer")?;
        self.token(TokenKind::LBrace)?;
        let mut binding = Binding::default();
        while !self.at(TokenKind::RBrace) {
            match self.word_text()?.as_str() {
                "store" => binding.store = Some(self.ident_statement()?),
                "key" => binding.key = Some(self.string_statement()?.value),
                "content_type" => binding.content_type = Some(self.string_statement()?.value),
                "metadata" => binding.metadata = Some(self.string_statement()?.value),
                "expires_in_ms" => binding.expires_in_ms = Some(self.number_statement()?),
                other => {
                    return Err(
                        self.error_previous(format!("unsupported transfer member '{other}'"))
                    )
                }
            }
        }
        self.token(TokenKind::RBrace)?;
        let end = self.token(TokenKind::RBrace)?.end;
        Ok(self.spanned(binding, start..end))
    }

    fn ident_list_statement(&mut self) -> miette::Result<Vec<Spanned<String>>> {
        self.token(TokenKind::LBracket)?;
        let mut values = Vec::new();
        while !self.at(TokenKind::RBracket) {
            values.push(self.ident()?);
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.token(TokenKind::RBracket)?;
        self.token(TokenKind::Semi)?;
        Ok(values)
    }

    fn string_list_statement(&mut self) -> miette::Result<Vec<String>> {
        self.token(TokenKind::LBracket)?;
        let mut values = Vec::new();
        while !self.at(TokenKind::RBracket) {
            values.push(self.string()?.value);
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }
        self.token(TokenKind::RBracket)?;
        self.token(TokenKind::Semi)?;
        Ok(values)
    }

    fn string_statement(&mut self) -> miette::Result<Spanned<String>> {
        let value = self.string()?;
        self.token(TokenKind::Semi)?;
        Ok(value)
    }

    fn ident_statement(&mut self) -> miette::Result<Spanned<String>> {
        let value = self.ident()?;
        self.token(TokenKind::Semi)?;
        Ok(value)
    }

    fn number_statement(&mut self) -> miette::Result<u64> {
        let span = self.token(TokenKind::Number)?;
        let value = self
            .text(&span)
            .parse()
            .map_err(|_| diagnostic(self.source, span.clone(), "integer value is too large"))?;
        self.token(TokenKind::Semi)?;
        Ok(value)
    }

    fn string(&mut self) -> miette::Result<Spanned<String>> {
        let span = self.token(TokenKind::String)?;
        let value = serde_json::from_str(self.text(&span)).map_err(|error| {
            diagnostic(
                self.source,
                span.clone(),
                format!("invalid string: {error}"),
            )
        })?;
        Ok(self.spanned(value, span))
    }

    fn ident(&mut self) -> miette::Result<Spanned<String>> {
        let span = self.token(TokenKind::Ident)?;
        Ok(self.spanned(self.text(&span).to_owned(), span))
    }

    fn word(&mut self, expected: &str) -> miette::Result<std::ops::Range<usize>> {
        let value = self.ident()?;
        if value.value != expected {
            return Err(self.error_at(&value, format!("expected '{expected}'")));
        }
        Ok(value.span)
    }

    fn word_text(&mut self) -> miette::Result<String> {
        Ok(self.ident()?.value)
    }

    fn token(&mut self, expected: TokenKind) -> miette::Result<std::ops::Range<usize>> {
        let Some(token) = self.tokens.get(self.position) else {
            return Err(self.error_here(format!("expected {}", token_name(&expected))));
        };
        if token.kind != expected {
            return Err(self.error_here(format!("expected {}", token_name(&expected))));
        }
        self.position += 1;
        Ok(token.span.clone())
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.tokens
            .get(self.position)
            .is_some_and(|token| token.kind == kind)
    }

    fn at_word(&self, word: &str) -> bool {
        self.tokens
            .get(self.position)
            .is_some_and(|token| token.kind == TokenKind::Ident && self.text(&token.span) == word)
    }

    fn done(&self) -> bool {
        self.position == self.tokens.len()
    }

    fn text(&self, span: &std::ops::Range<usize>) -> &str {
        &self.source.text[span.clone()]
    }

    fn current_span(&self) -> std::ops::Range<usize> {
        self.tokens
            .get(self.position)
            .map(|token| token.span.clone())
            .unwrap_or(self.source.text.len()..self.source.text.len())
    }

    fn previous_span(&self) -> std::ops::Range<usize> {
        self.tokens[self.position - 1].span.clone()
    }

    fn spanned<T>(&self, value: T, span: std::ops::Range<usize>) -> Spanned<T> {
        Spanned {
            value,
            source: self.source_index,
            span,
        }
    }

    fn error_here(&self, message: impl Into<String>) -> Report {
        diagnostic(self.source, self.current_span(), message)
    }

    fn error_previous(&self, message: impl Into<String>) -> Report {
        diagnostic(self.source, self.previous_span(), message)
    }

    fn error_at<T>(&self, value: &Spanned<T>, message: impl Into<String>) -> Report {
        diagnostic(self.source, value.span.clone(), message)
    }
}

fn insert<T>(
    map: &mut BTreeMap<String, Spanned<T>>,
    name: Spanned<String>,
    value: Spanned<T>,
    source: &Source,
) -> miette::Result<()> {
    match map.entry(name.value.clone()) {
        Entry::Vacant(entry) => {
            entry.insert(value);
            Ok(())
        }
        Entry::Occupied(_) => Err(diagnostic(
            source,
            name.span,
            format!("duplicate declaration '{}'", name.value),
        )),
    }
}

pub(crate) fn diagnostic(
    source: &Source,
    span: std::ops::Range<usize>,
    message: impl Into<String>,
) -> Report {
    Report::new(
        MietteDiagnostic::new(message.into()).with_labels(vec![LabeledSpan::underline((
            span.start,
            span.len().max(1),
        ))]),
    )
    .with_source_code(NamedSource::new(
        source.path.display().to_string(),
        source.text.clone(),
    ))
}

fn token_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Ident => "an identifier",
        TokenKind::Number => "an integer",
        TokenKind::String => "a quoted string",
        TokenKind::LBrace => "'{'",
        TokenKind::RBrace => "'}'",
        TokenKind::LParen => "'('",
        TokenKind::RParen => "')'",
        TokenKind::LBracket => "'['",
        TokenKind::RBracket => "']'",
        TokenKind::LAngle => "'<'",
        TokenKind::RAngle => "'>'",
        TokenKind::Colon => "':'",
        TokenKind::Semi => "';'",
        TokenKind::Eq => "'='",
        TokenKind::Comma => "','",
        TokenKind::Question => "'?'",
        TokenKind::Pipe => "'|'",
    }
}
