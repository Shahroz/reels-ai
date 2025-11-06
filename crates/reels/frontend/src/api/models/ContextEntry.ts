/* generated using openapi-typescript-codegen -- do not edit */
/* istanbul ignore file */
/* tslint:disable */
/* eslint-disable */
/**
 * A piece of persisted context.
 */
export type ContextEntry = {
    /**
     * The textual content of the context entry.
     */
    content: string;
    /**
     * Optional identifier for where this context originated (e.g., URL, file).
     */
    source?: string | null;
    /**
     * Timestamp indicating when this context entry was created or recorded.
     */
    timestamp: string;
};

