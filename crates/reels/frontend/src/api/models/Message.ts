/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { Attachment } from './Attachment';
/**
 * Represents a message with a role (sender) and content.
 */
export type Message = {
    /**
     * Optional attachments associated with the message.
     */
    attachments?: Array<Attachment>;
    /**
     * The textual content of the message.
     */
    content: string;
    /**
     * The role indicating the sender (user, assistant, system).
     */
    role: string;
};

