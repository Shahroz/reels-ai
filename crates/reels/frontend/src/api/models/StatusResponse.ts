/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { SessionStatus } from './SessionStatus';
/**
 * Response payload for the session status endpoint.
 */
export type StatusResponse = {
    /**
     * The unique identifier of the session.
     */
    session_id: string;
    /**
     * The current status of the session.
     */
    status: SessionStatus;
    /**
     * Optional remaining time until the session expires, in seconds.
     * Uses a custom serializer/deserializer if needed (assumed crate::utils::serde_option_duration_as_secs exists).
     */
    time_remaining?: string | null;
};

