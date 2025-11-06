/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { Message } from '../models/Message';
import type { ResearchRequest } from '../models/ResearchRequest';
import type { StatusResponse } from '../models/StatusResponse';
import type { CancelablePromise } from '../core/CancelablePromise';
import { OpenAPI } from '../core/OpenAPI';
import { request as __request } from '../core/request';
export class LoupeService {
    /**
     * Handles POST requests to start a research session.
     * # Arguments
     *
     * * `request_payload` - JSON containing the user's research instruction.
     * * `app_state` - Shared application state containing session storage.
     *
     * # Returns
     *
     * * `HttpResponse` - OK (200) with JSON body `{"session_id": "..."}` on success,
     * or InternalServerError (500) if session creation or history update fails.
     * The research process itself is started asynchronously in the background.
     * @returns any Research session started successfully
     * @throws ApiError
     */
    public static startResearch({
        requestBody,
    }: {
        requestBody: ResearchRequest,
    }): CancelablePromise<any> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/loupe/research',
            body: requestBody,
            mediaType: 'application/json',
            errors: {
                500: `Internal server error starting session or adding initial entry`,
            },
        });
    }
    /**
     * Handles POST requests to send a message to a session and update session state.
     * @returns any Message posted successfully
     * @throws ApiError
     */
    public static postMessage({
        sessionId,
        requestBody,
    }: {
        /**
         * ID of the session to post the message to
         */
        sessionId: string,
        requestBody: Message,
    }): CancelablePromise<any> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/loupe/session/{session_id}/message',
            path: {
                'session_id': sessionId,
            },
            body: requestBody,
            mediaType: 'application/json',
            errors: {
                404: `Session not found`,
            },
        });
    }
    /**
     * Handles GET requests to query session status.
     * Expects SessionId in the path and AppState via web::Data.
     * @returns StatusResponse Session status retrieved successfully
     * @throws ApiError
     */
    public static getStatus({
        sessionId,
    }: {
        /**
         * ID of the session to query
         */
        sessionId: string,
    }): CancelablePromise<StatusResponse> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/loupe/session/{session_id}/status',
            path: {
                'session_id': sessionId,
            },
            errors: {
                404: `Session not found`,
            },
        });
    }
    /**
     * Handles GET requests to establish a WebSocket conversation stream.
     * Upgrades the connection to WebSocket and starts the `WsSession` actor.
     * Requires `SessionId` from the path and `AppState` from application data.
     * @returns void
     * @throws ApiError
     */
    public static conversationStream({
        sessionId,
    }: {
        /**
         * ID of the session to stream events for
         */
        sessionId: string,
    }): CancelablePromise<void> {
        return __request(OpenAPI, {
            method: 'GET',
            url: '/loupe/session/{session_id}/stream',
            path: {
                'session_id': sessionId,
            },
            errors: {
                400: `Bad request during WebSocket handshake`,
                500: `Internal server error during WebSocket handshake`,
            },
        });
    }
    /**
     * Handles POST requests to terminate a session.
     * Extracts the SessionId from the path and the AppState from application data.
     * Removes the session from the shared state maps and notifies WebSocket clients.
     * @returns any Session terminated (or did not exist)
     * @throws ApiError
     */
    public static terminateSession({
        sessionId,
    }: {
        /**
         * ID of the session to terminate
         */
        sessionId: string,
    }): CancelablePromise<any> {
        return __request(OpenAPI, {
            method: 'POST',
            url: '/loupe/session/{session_id}/terminate',
            path: {
                'session_id': sessionId,
            },
        });
    }
}
