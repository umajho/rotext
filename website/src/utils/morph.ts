export function morphAttributes(oldEl: HTMLElement, newEl: HTMLElement) {
  const newAttrNames: Set<string> = new Set();
  for (const attr of newEl.attributes) {
    newAttrNames.add(attr.name);
    oldEl.setAttribute(attr.name, attr.value);
  }
  for (const attrName of [...oldEl.attributes].map((attr) => attr.name)) {
    if (!newAttrNames.has(attrName)) {
      oldEl.removeAttribute(attrName);
    }
  }
}
