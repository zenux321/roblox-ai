import styles from './Avatar.module.css';

interface PersonAvatarProps {
  initial: string;
  palette: 'self' | 'peer';
  size?: 20 | 30 | 32;
  online?: boolean;
}

export function PersonAvatar({ initial, palette, size = 30, online }: PersonAvatarProps) {
  return (
    <span className={`${styles.person} ${styles[palette]} ${styles[`s${size}`]}`} aria-hidden="true">
      {initial}
      {online !== undefined && <span className={`${styles.presence} ${online ? styles.online : ''}`} />}
    </span>
  );
}

interface AiAvatarProps {
  size?: 20 | 30;
  soft?: boolean;
}

export function AiAvatar({ size = 30, soft = false }: AiAvatarProps) {
  return (
    <span className={`${styles.ai} ${soft ? styles.aiSoft : ''} ${styles[`s${size}`]}`} aria-hidden="true">
      AI
    </span>
  );
}
