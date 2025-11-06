/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
import type { FullToolResponse } from './FullToolResponse';
import type { UserToolFailure } from './UserToolFailure';
/**
 * Represents the outcome of a tool execution.
 */
export type ToolResponse = ({
    /**
     * Indicates a successful tool execution, containing the full response.
     */
    Success: FullToolResponse;
} | {
    /**
     * Indicates a failed tool execution, containing details about the failure.
     */
    Failure: UserToolFailure;
});

