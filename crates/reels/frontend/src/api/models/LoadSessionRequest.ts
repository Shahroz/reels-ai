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
 * Payload for the POST /session/load endpoint.
 *
 * Contains the state needed to recreate a session. The server will generate
 * a new session_id and timestamps upon loading.
 */
export type LoadSessionRequest = {
    /**
     * Configuration parameters applied to this session.
     */
    config: SessionConfig;
    /**
     * Collection of context snippets gathered during the session.
     */
    context: Array<ContextEntry>;
    /**
     * Ordered list of conversation entries (user, agent, tool messages).
     */
    history: Array<ConversationEntry>;
    /**
     * Chronological list of messages exchanged directly for prompt building.
     */
    messages: Array<Message>;
    /**
     * The primary research goal or objective for the session.
     */
    research_goal?: string | null;
    /**
     * Current status of the session (e.g., Pending, InProgress).
     */
    status: SessionStatus;
    /**
     * Optional system message to guide the assistant.
     */
    system_message?: string | null;
};

