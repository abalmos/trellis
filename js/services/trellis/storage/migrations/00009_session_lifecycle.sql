ALTER TABLE `sessions` ADD `status` text DEFAULT 'active' NOT NULL;
--> statement-breakpoint
ALTER TABLE `sessions` ADD `ended_at` text;
--> statement-breakpoint
ALTER TABLE `sessions` ADD `ended_reason` text;
