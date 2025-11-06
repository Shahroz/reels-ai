/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { Attachment } from './Attachment';
import type { Sender } from './Sender';
import type { ToolChoice } from './ToolChoice';
import type { ToolResponse } from './ToolResponse';
/**
 * A single entry in the conversation stream.
 */
export type ConversationEntry = {
    /**
     * Attachments associated with this entry.
     */
    attachments: Array<Attachment>;
    /**
     * Depth of the entry in the conversation tree.
     */
    depth: number;
    /**
     * Unique identifier for this entry.
     */
    id: string;
    /**
     * The textual content of the message.
     */
    message: string;
    /**
     * Identifier of the parent entry in a threaded conversation, if any.
     */
    parent_id?: string | null;
    /**
     * Who sent the message (User, Agent, or Tool).
     */
    sender: Sender;
    /**
     * Timestamp when the entry was created. Uses fully qualified path.
     */
    timestamp: string;
    tool_choice?: (null | ToolChoice);
    tool_response?: (null | ToolResponse);
    /**
     * Tools selected by the agent relevant to this entry. Uses fully qualified path.
     */
    tools: Array<ToolChoice>;
};

