/**
 * ThemeProvider Component
 *
 * Manages the application theme (light, dark, system) and applies it.
 * Provides theme context to descendant components.
 * Based on common patterns like next-themes and shadcn/ui theme provider.
 *
 * Revision History:
 * - 2025-05-03T16:43:05Z @AI: Initial implementation to fix build error and missing export.
 */
import React, { createContext, useContext, useEffect, useState } from 'react';

type Theme = 'dark' | 'light' | 'system';

interface ThemeProviderProps {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
  attribute?: string; // e.g., 'class' or 'data-theme'
  enableSystem?: boolean;
}

interface ThemeProviderState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

const initialState: ThemeProviderState = {
  theme: 'system',
  setTheme: () => null,
};

const ThemeProviderContext = createContext<ThemeProviderState>(initialState);

export function ThemeProvider({
  children,
  defaultTheme = 'system',
  storageKey = 'vite-ui-theme', // Changed storage key slightly
  attribute = 'class', // Default to class for Tailwind compatibility
  enableSystem = true, // Default to enable system theme detection
}: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>(() => {
    try {
        const storedTheme = localStorage.getItem(storageKey);
        if (storedTheme && ['light', 'dark', 'system'].includes(storedTheme)) {
            return storedTheme as Theme;
        }
    } catch (e) {
        console.error("Failed to access localStorage for theme", e);
    }
    return defaultTheme;
  });

  useEffect(() => {
    const root = window.document.documentElement;

    root.classList.remove('light', 'dark');

    let systemTheme: Theme = 'light'; // Default system theme if detection fails or is disabled
    if (enableSystem && theme === 'system') {
       try {
           systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches
            ? 'dark'
            : 'light';
       } catch(e) {
           console.error("Failed to detect system theme preference", e);
           systemTheme = 'light'; // Fallback
       }
    }

    const effectiveTheme = theme === 'system' ? systemTheme : theme;

    if (attribute === 'class') {
        root.classList.add(effectiveTheme);
    } else {
        root.setAttribute(attribute, effectiveTheme);
    }

    try {
        localStorage.setItem(storageKey, theme);
    } catch (e) {
        console.error("Failed to save theme to localStorage", e);
    }

  }, [theme, storageKey, attribute, enableSystem]);

  const value = {
    theme,
    setTheme: (newTheme: Theme) => {
      if (['light', 'dark', 'system'].includes(newTheme)) {
          setTheme(newTheme);
      } else {
          console.warn(`Invalid theme value provided: ${newTheme}`);
      }
    },
  };

  return (
    <ThemeProviderContext.Provider value={value}>
      {children}
    </ThemeProviderContext.Provider>
  );
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext);

  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }

  return context;
};
