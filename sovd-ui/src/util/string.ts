export function formatToTitleCase(str: string, separator = '_'): string {
    return str
        .replace(/([A-Z])/g, ' $1')
        .replace(new RegExp(separator, 'g'), ' ')
        .split(' ')
        .filter((word) => word)
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ')
        .trim();
}
