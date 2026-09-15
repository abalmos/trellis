# Verify real exported parentage and durable links without printing span attributes.
def outcome: [.attributes[]? | select(.key == "trellis.outcome") | .value.stringValue] | first;
[.[].resourceSpans[]
 | ([.resource.attributes[] | select(.key == "service.name") | .value.stringValue] | first) as $service
 | .scopeSpans[].spans[]
 | {service: $service, name, traceId, spanId, parentSpanId, links, attributes}]
as $spans
| {
    browserToRust: any($spans[];
      .service == "trellis-server" and .name == "trellis.rpc.server" and
      (. as $child | any($spans[];
        .service == "trellis-browser-console" and .name == "trellis.rpc.client" and
        .traceId == $child.traceId and .spanId == $child.parentSpanId))),
    rustToTypeScript: any($spans[];
      .service == "provider" and .name == "trellis.rpc.server" and
      (. as $child | any($spans[];
        .service == "runtime-rust-caller" and .name == "trellis.rpc.client" and
        .traceId == $child.traceId and .spanId == $child.parentSpanId))),
    durableOperation: any($spans[];
      .service == "runtime-rust-provider" and .name == "trellis.operation.execute.start" and
      (. as $execution | any($execution.links[]?;
        . as $link | any($spans[];
          .service == "runtime-trellis.OperationCaller" and
          .traceId == $link.traceId and .spanId == $link.spanId)))),
    typeScriptOperation: any($spans[];
      .service == "provider" and .name == "trellis.operation.execute.start" and
      (. as $execution | any($execution.links[]?;
        . as $link | any($spans[];
          .name == "trellis.rpc.client" and
          .traceId == $link.traceId and .spanId == $link.spanId)))),
    durableJob: any($spans[];
      .service == "provider" and .name == "trellis.job.attempt" and outcome == "retry" and
      (. as $retry | any($retry.links[]?;
        . as $link | any($spans[];
          .service == "provider" and .name == "trellis.rpc.server" and
          .traceId == $link.traceId and .spanId == $link.spanId) and
        any($spans[];
          .service == "provider" and .name == "trellis.job.attempt" and
          outcome == "completed" and
          any(.links[]?; .traceId == $link.traceId and .spanId == $link.spanId))))),
    noSecretAttributes: all($spans[];
      all(.attributes[]?; (.key | test("password|proof|secret|token|cookie|credential|payload|body|message|authorization|nats\\.subject"; "i") | not)))
  }
| if all(.[]; .) then . else error("collected trace evidence failed validation") end
