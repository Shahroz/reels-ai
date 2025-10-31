/**
 * ResearchItemDisplayCard Component
 *
 * This component displays a single research or document item in a card format,
 * allowing for selection. It shows the item's title and a content snippet.
 *
 * Revision History:
 * - 2025-05-13T14:32:19Z @AI: Initial creation of the ResearchItemDisplayCard component.
 */

import React from 'react';

const ResearchItemDisplayCard = ({ item, isSelected, onSelect }) => {
  const baseClasses = "p-3 rounded-lg shadow-sm cursor-pointer hover:shadow-md transition-all duration-150 ease-in-out mb-2";
  const selectedClasses = "bg-indigo-100 dark:bg-indigo-700 border border-indigo-500 dark:border-indigo-400";
  const unselectedClasses = "bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500";

  // Ensure item and item.content exist before trying to access properties or methods
  const title = item && item.title ? item.title : 'Untitled';
  let contentSnippet = item && typeof item.content === 'string' ? item.content : '';
  if (contentSnippet.length > 80) {
    contentSnippet = contentSnippet.substring(0, 77) + "...";
  }

  const handleSelect = () => {
    if (item && item.id) {
      onSelect(item.id);
    }
  };

  return (
    <div
      className={`${baseClasses} ${isSelected ? selectedClasses : unselectedClasses}`}
      onClick={handleSelect}
      role="button"
      tabIndex={0}
      onKeyPress={(e) => { if (e.key === 'Enter' || e.key === ' ') handleSelect(); }}
      aria-pressed={isSelected}
    >
      <h3 className={`font-semibold text-sm mb-1 truncate ${isSelected ? 'text-indigo-800 dark:text-indigo-100' : 'text-gray-800 dark:text-gray-100'}`}>
        {title}
      </h3>
      <p className={`text-xs ${isSelected ? 'text-indigo-700 dark:text-indigo-200' : 'text-gray-600 dark:text-gray-300'}`}>
        {contentSnippet}
      </p>
    </div>
  );
};

export default ResearchItemDisplayCard;