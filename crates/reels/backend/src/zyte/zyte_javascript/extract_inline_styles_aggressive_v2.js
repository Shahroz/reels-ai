(function() {
    const elements = document.getElementsByTagName('*');

    // Cache for computed styles
    const styleCache = new WeakMap();

    // Common browser defaults to skip
    const commonDefaults = {
        // 'display': ['block', 'inline'],
        // 'margin': ['0px'],
        // 'padding': ['0px'],
        // 'border': ['0px solid rgb(0, 0, 0)'],
        // 'border-top': ['0px none rgb(0, 0, 0)'],
        // 'border-right': ['0px none rgb(0, 0, 0)'],
        // 'border-bottom': ['0px none rgb(0, 0, 0)'],
        // 'border-left': ['0px none rgb(0, 0, 0)']
    };

    // List of properties that we want to always preserve
    const importantProperties = ['color', 'background-color'];

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

    // FIRST PASS: Optimize inline styles by reapplying only those properties that actually differ
    Array.from(elements).reverse().forEach(element => {
        const tag = element.tagName.toLowerCase();
        if (tag === 'script' || tag === 'style') return;
        if (!element.style) return;

        const computedStyle = window.getComputedStyle(element);
        const optimizedStyles = {};
        for (let property of computedStyle) {
            try {
                const value = computedStyle.getPropertyValue(property);
                if (isStyleDifferent(element, property, value)) {
                    optimizedStyles[property] = value;
                }
            } catch (e) {
                console.warn(`Couldn't process property ${property} on element`, element);
            }
        }
        element.removeAttribute('style');
        element.removeAttribute('class');
        for (const [property, value] of Object.entries(optimizedStyles)) {
            element.style.setProperty(property, value);
        }
    });

    // SECOND PASS: For each inline style, test whether its removal changes computed style.
    // If removal affects computed style, then it's essential so reapply it.
    Array.from(elements).forEach(element => {
        if (!element.style) return;
        const properties = Array.from(element.style);
        properties.forEach(property => {
            if (importantProperties.includes(property)) return; // Skip whitelisted properties

            const originalValue = element.style.getPropertyValue(property);
            const computedWith = window.getComputedStyle(element).getPropertyValue(property);
            element.style.removeProperty(property);
            element.offsetHeight; // Force reflow
            const computedWithout = window.getComputedStyle(element).getPropertyValue(property);
            if (computedWith !== computedWithout) {
                element.style.setProperty(property, originalValue);
            }
        });
    });

    // --- Preserve @font-face and CSS variable (custom property) declarations ---
    const preservedCSSRules = [];
    document.querySelectorAll('style').forEach(styleEl => {
        try {
            const sheet = styleEl.sheet;
            if (sheet && sheet.cssRules) {
                for (const rule of sheet.cssRules) {
                    // Preserve @font-face rules
                    if (rule.type === CSSRule.FONT_FACE_RULE) {
                        preservedCSSRules.push(rule.cssText);
                    }
                    // Preserve :root rules that contain CSS variable definitions
                    else if (rule.selectorText === ':root') {
                        if (rule.cssText.indexOf('--') !== -1) {
                            preservedCSSRules.push(rule.cssText);
                        }
                    }
                }
            }
        } catch (e) {
            // Fallback for cases where accessing cssRules might be restricted
            const textContent = styleEl.textContent;
            // Extract @font-face blocks
            const fontFaceMatches = textContent.match(/@font-face\s*\{[^}]+\}/g);
            if (fontFaceMatches) {
                preservedCSSRules.push(...fontFaceMatches);
            }
            // Extract :root blocks containing CSS custom properties
            const rootMatches = textContent.match(/:root\s*\{[^}]+\}/g);
            if (rootMatches) {
                rootMatches.forEach(match => {
                    if (match.indexOf('--') !== -1) {
                        preservedCSSRules.push(match);
                    }
                });
            }
        }
    });

    // Remove all <style> and <script> tags from the document
    document.querySelectorAll('style, script').forEach(el => el.remove());

    // If we have any preserved CSS rules, create a new <style> element and append them
    if (preservedCSSRules.length > 0) {
        const newStyleEl = document.createElement('style');
        newStyleEl.textContent = preservedCSSRules.join('\n');
        document.head.appendChild(newStyleEl);
    }

    console.log('Optimized inline styles applied, custom fonts and CSS variables preserved, and all <style> and <script> tags removed.');
})();
