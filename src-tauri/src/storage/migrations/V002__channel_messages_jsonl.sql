-- V002__channel_messages_jsonl.sql
-- Add messages_jsonl_path column to channels table.
-- Channel message bodies will be stored in independent JSONL files,
-- while channel JSON only retains metadata.

ALTER TABLE channels ADD COLUMN messages_jsonl_path TEXT;
