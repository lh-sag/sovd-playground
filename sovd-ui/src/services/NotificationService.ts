import { ref } from 'vue';
import { CONSTANTS } from '../constants';
import { Level } from './TelemetryService';

export interface Notification {
    title: string;
    message: string;
    level: Level;
}

export class NotificationService {
    currentToast = ref<Notification | null>(null);

    showToast(title: string, message: string, level: Level = Level.INFO) {
        this.currentToast.value = { title, message, level };

        setTimeout(() => {
            this.currentToast.value = null;
        }, CONSTANTS.TOAST_TIMEOUT);
    }
}

export const notificationService = new NotificationService();
