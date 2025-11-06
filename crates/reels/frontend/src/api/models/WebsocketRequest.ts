/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { Attachment } from './Attachment';
/**
 * Messages sent by clients to the AgentLoop service via WebSocket.
 */
export type WebsocketRequest = ({
    /**
     * Represents input or instructions provided by the user during a session.
     */
    UserInput: {
        attachments?: Array<Attachment>;
        instruction: string;
    };
} | 'Interrupt');

