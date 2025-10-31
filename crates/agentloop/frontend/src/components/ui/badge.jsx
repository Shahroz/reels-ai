/**
 * Badge Component
 *
 * Displays a badge or a component that looks like a badge.
 * Copied from narrativ/frontend and converted to JSX.
 *
 * Revision History:
 * - 2025-05-13T14:22:24Z @AI: Initial creation and conversion to JSX.
 */
import React from 'react';
import { cva } from 'class-variance-authority';
import { cn } from '../../lib/utils.js';

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
  {
    variants: {
      variant: {
        default: "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
        secondary: "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
        destructive: "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
        outline: "text-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
);

/**
 * @param {object} props
 * @param {string} [props.className]
 * @param {('default'|'secondary'|'destructive'|'outline')} [props.variant]
 * @param {React.HTMLAttributes<HTMLDivElement>} ...otherProps
 */
function Badge({ className, variant, ...props }) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}

export { Badge, badgeVariants };