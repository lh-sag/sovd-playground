export function parseJWT(token: string) {
    try {
        if (!token || typeof token !== 'string') {
            throw new Error('Invalid token type');
        }
        const parts = token.split('.');
        if (parts.length !== 3) {
            throw new Error('Invalid JWT format');
        }
        const payload = parts[1];
        const decoded = atob(payload.replaceAll('-', '+').replaceAll('_', '/'));
        return JSON.parse(decoded);
    } catch (e) {
        console.error('Failed to parse JWT:', e);
        return null;
    }
}
