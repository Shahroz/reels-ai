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

    // FIRST PASS:
    // Optimize inline styles by preserving only properties that differ from inherited values.
    Array.from(elements).reverse().forEach(element => {
        const tag = element.tagName.toLowerCase();
        // Skip script and style tags from processing (we'll remove them later)
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

        // Remove inline styles and classes
        element.removeAttribute('style');
        element.removeAttribute('class');

        // Reapply the optimized inline styles
        for (const [property, value] of Object.entries(optimizedStyles)) {
            element.style.setProperty(property, value);
        }
    });

    // SECOND PASS:
    // For each inline style property, temporarily remove it and check if the computed style changes.
    // Skip any properties in the importantProperties list.
    Array.from(elements).forEach(element => {
        if (!element.style) return;
        const properties = Array.from(element.style);
        properties.forEach(property => {
            if (importantProperties.includes(property)) return; // Always preserve these

            const originalValue = element.style.getPropertyValue(property);
            const computedWith = window.getComputedStyle(element).getPropertyValue(property);

            // Remove the inline property and force a reflow.
            element.style.removeProperty(property);
            element.offsetHeight; // forces reflow

            const computedWithout = window.getComputedStyle(element).getPropertyValue(property);

            // If removal changes computed style, the property is essential; reapply it.
            if (computedWith !== computedWithout) {
                element.style.setProperty(property, originalValue);
            }
        });
    });

    // FINAL STEP:
    // Remove all <style> and <script> tags from the document (including those in the <head>).
    document.querySelectorAll('style, script').forEach(el => el.remove());

    console.log('Optimized inline styles applied and all <style> and <script> tags have been removed.');
})();
