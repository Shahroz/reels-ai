/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { LoadSessionRequest } from '../models/LoadSessionRequest';
import type { SessionData } from '../models/SessionData';
import type { CancelablePromise } from '../core/CancelablePromise';
import { OpenAPI } from '../core/OpenAPI';
import { request as __request } from '../core/request';
export class SessionManagementService {
    /**
     * Loads a session state from the request body.
     * Path: POST /session/load
     *
     * Accepts a `LoadSessionRequest` JSON object in the request body.
     * Creates a new session based on the provided state, assigns a new unique
     * session ID, and stores it in the application state. Returns the new
     * session ID upon successful creation.
     * @returns string Session loaded successfully, returns new session ID
     * @throws ApiError
     */
    public static loadSessionState({
        requestBody,
    }: {
        requestBody: LoadSessionRequest,
    }): CancelablePromise<string> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/session/load',
            body: requestBody,
            mediaType: 'application/json',
            errors: {
                400: `Invalid request body format`,
            },
        });
    }
    /**
     * Retrieves the full state of a specific session.
     * Path: GET /session/{session_id}/state
     *
     * Returns the complete `SessionData` object for the specified session ID
     * as a JSON response. Returns 404 if the session is not found.
     * @returns SessionData Session state retrieved successfully
     * @throws ApiError
     */
    public static getSessionState({
        sessionId,
    }: {
        /**
         * Unique identifier of the session
         */
        sessionId: string,
    }): CancelablePromise<SessionData> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/session/{session_id}/state',
            path: {
                'session_id': sessionId,
            },
            errors: {
                404: `Session not found`,
            },
        });
    }
}
