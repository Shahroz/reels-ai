/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
/**
 * Represents the status of an agent session.
 */
export type SessionStatus = ('Pending' | {
    /**
     * The session is actively processing the user request.
     * Includes optional progress information.
     */
    Running: {
        progress?: string | null;
    };
} | 'Completed' | 'Error' | 'AwaitingInput' | 'Interrupted' | 'Timeout');

