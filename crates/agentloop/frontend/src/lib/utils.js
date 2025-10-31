/**
 * Utility Functions
 *
 * Provides utility functions for the application, such as class name joining and date humanization.
 * This file was copied from narrativ/frontend/src/lib/utils.ts and converted to JavaScript.
 *
 * Revision History:
 * - 2025-05-13T14:22:24Z @AI: Copied from narrativ/frontend and converted to JavaScript.
 */

import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { formatDistanceToNow, parseISO, isValid } from 'date-fns';

/**
 * Combines class names using clsx and tailwind-merge.
 * @param {...any} inputs - Class names or conditions.
 * @returns {string} The merged class names.
 */
export function cn(...inputs) {
  return twMerge(clsx(inputs));
}

/**
 * Humanizes a date string into a relative time format (e.g., "2 days ago").
 * Handles invalid date strings gracefully.
 * @param {string} dateString - An ISO 8601 date string.
 * @returns {string} A human-readable date string or 'Invalid date' on error.
 */
export function humanizeDate(dateString) {
  try {
    const date = parseISO(dateString);
    if (!isValid(date)) {
      throw new Error('Invalid date value');
    }
    // Return relative time (e.g., "about 2 hours ago", "2 days ago")
    return formatDistanceToNow(date, { addSuffix: true });
  } catch (error) {
    console.error(`Error parsing or formatting date '${dateString}':`, error);
    return 'Invalid date';
  }
}