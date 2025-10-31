(function() {
    // Core selectors to analyze
    const selectors = [
        'body', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
        'p', 'a', 'button', 'input', 'textarea'
    ];

    // Properties to extract, focusing on fonts and key styles
    const propertiesToExtract = [
        'background-color', 'color', 'font-size', 'font-family',
        'font-weight', 'font-style', 'text-align', 'padding', 'margin',
        'text-decoration', 'border-width', 'border-style', 'border-color',
        'border-radius', 'display', 'cursor', 'box-shadow', 'transition', // Added for button effects
        'opacity', 'pointer-events' // For interactivity
    ];

    const styleMap = {};

    // Helper function to check if an element is visible
    function isVisible(element) {
        const style = window.getComputedStyle(element);
        return style.display !== 'none' && style.visibility !== 'hidden' && style.opacity !== '0';
    }

    // Function to extract styles from an element and its first relevant child variation
    function extractStyles(element, selector) {
        if (!element || !isVisible(element)) return;

        const computedStyle = window.getComputedStyle(element);
        const styleObj = {};
        propertiesToExtract.forEach(prop => {
            styleObj[prop] = computedStyle.getPropertyValue(prop);
        });

        // Store the style only if this selector hasn't been processed yet
        if (!styleMap[selector]) {
            styleMap[selector] = {
                example: styleObj,
                variation: null // For a single child variation (e.g., emphasis)
            };
        }

        // Look for a single child with a different font style (e.g., emphasis)
        const children = Array.from(element.children);
        for (const child of children) {
            if (!isVisible(child)) continue;

            const childComputedStyle = window.getComputedStyle(child);
            const childStyleObj = {};
            propertiesToExtract.forEach(prop => {
                childStyleObj[prop] = childComputedStyle.getPropertyValue(prop);
            });

            // Check if the child has a different font-related style
            if (
                childStyleObj['font-family'] !== styleObj['font-family'] ||
                childStyleObj['font-weight'] !== styleObj['font-weight'] ||
                childStyleObj['font-style'] !== styleObj['font-style'] ||
                childStyleObj['font-size'] !== styleObj['font-size']
            ) {
                styleMap[selector].variation = {
                    tagName: child.tagName.toLowerCase(),
                    className: child.className || null,
                    style: childStyleObj
                };
                break; // Stop after finding the first variation
            }
        }
    }

    // Analyze elements for each selector
    selectors.forEach(selector => {
        const elements = document.querySelectorAll(selector);
        if (elements.length === 0) return;

        if (selector === 'a') {
            // Handle <a> elements separately to differentiate normal links and buttons
            let normalLinkProcessed = false;
            let buttonLinkProcessed = false;

            for (const element of elements) {
                if (!isVisible(element)) continue;

                const className = element.className.toLowerCase();
                const isButtonLike = className.includes('button') || className.includes('btn');

                if (isButtonLike && !buttonLinkProcessed) {
                    extractStyles(element, 'a.button');
                    buttonLinkProcessed = true;
                } else if (!isButtonLike && !normalLinkProcessed) {
                    extractStyles(element, 'a');
                    normalLinkProcessed = true;
                }

                // Stop if we've processed both a normal link and a button link
                if (normalLinkProcessed && buttonLinkProcessed) break;
            }
        } else {
            // For non-<a> elements, process the first visible instance
            for (const element of elements) {
                if (isVisible(element)) {
                    extractStyles(element, selector);
                    break;
                }
            }
        }
    });

    // Create and append result div
    const resultDiv = document.createElement('div');
    resultDiv.id = 'extracted-styles';
    resultDiv.style.display = 'none';
    resultDiv.textContent = JSON.stringify(styleMap, null, 2);
    document.body.appendChild(resultDiv);
})();