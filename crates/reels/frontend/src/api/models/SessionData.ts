/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { ContextEntry } from './ContextEntry';
import type { ConversationEntry } from './ConversationEntry';
import type { Message } from './Message';
import type { SessionConfig } from './SessionConfig';
import type { SessionStatus } from './SessionStatus';
/**
 * In-memory representation of a research session.
 *
 * Holds the current state, configuration, history, context, messages,
 * creation time, and last activity time for an ongoing research session.
 */
export type SessionData = {
    /**
     * Configuration parameters applied to this session.
     */
    config: SessionConfig;
    /**
     * Collection of context snippets gathered during the session.
     */
    context: Array<ContextEntry>;
    /**
     * The primary research goal or objective for the session.
     * Timestamp indicating when the session was initiated.
     */
    created_at: string;
    /**
     * Ordered list of conversation entries (user, agent, tool messages).
     */
    history: Array<ConversationEntry>;
    /**
     * Timestamp indicating the last recorded activity within the session.
     */
    last_activity_timestamp: string;
    /**
     * Chronological list of messages exchanged directly for prompt building.
     */
    messages: Array<Message>;
    /**
     * Current status of the session (e.g., Pending, InProgress).
     * The primary research goal or objective for the session. Updated by user messages.
     */
    research_goal?: string | null;
    /**
     * Unique identifier for the session.
     */
    session_id: string;
    status: SessionStatus;
    /**
     * Optional system message to guide the assistant.
     */
    system_message?: string | null;
};

