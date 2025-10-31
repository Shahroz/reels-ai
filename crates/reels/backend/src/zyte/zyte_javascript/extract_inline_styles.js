(function() {
    const elements = document.getElementsByTagName('*');

    // Cache for computed styles
    const styleCache = new WeakMap();

    // Common browser defaults to skip
    const commonDefaults = {
        'display': ['block', 'inline'],
        'margin': ['0px'],
        'padding': ['0px'],
        'border': ['0px solid rgb(0, 0, 0)']
    };

    function isStyleDifferent(element, property, value) {
        if (commonDefaults[property]?.includes(value)) {
            return false;
        }

        let currentElement = element.parentElement;

        while (currentElement) {
            let parentStyle = styleCache.get(currentElement);
            if (!parentStyle) {
                parentStyle = window.getComputedStyle(currentElement);
                styleCache.set(currentElement, parentStyle);
            }

            if (parentStyle.getPropertyValue(property) !== value) {
                return true;
            }

            currentElement = currentElement.parentElement;
        }

        return false;
    }

    Array.from(elements).reverse().forEach(element => {
        // Skip script and style tags
        if (element.tagName.toLowerCase() === 'script' ||
            element.tagName.toLowerCase() === 'style') {
            return;
        }

        if (!element.classList || !element.style) return;

        const computedStyle = window.getComputedStyle(element);
        const inlineStyles = {};

        for (let property of computedStyle) {
            const value = computedStyle.getPropertyValue(property);
            if (isStyleDifferent(element, property, value)) {
                inlineStyles[property] = value;
            }
        }

        Object.entries(inlineStyles).forEach(([property, value]) => {
            element.style[property] = value;
        });

        element.removeAttribute('class');
    });

    console.log('Classes removed and optimized inline styles applied (skipping script/style tags)!');
})();