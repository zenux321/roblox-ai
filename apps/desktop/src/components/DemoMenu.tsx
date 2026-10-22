import { usePopover } from '../app/usePopover';
import { t } from '../i18n';
import styles from './DemoMenu.module.css';

export type DemoAction = 'normal' | 'loading' | 'empty' | 'model_missing';

export function DemoMenu({
  onAction,
  providerMissing,
}: {
  onAction: (a: DemoAction) => void;
  providerMissing: boolean;
}) {
  const { open: menuOpen, toggle: menuToggle, close: menuClose, anchor: menuAnchor } = usePopover();
  const items: { action: DemoAction; label: string }[] = [
    { action: 'normal', label: t('demo.normal') },
    { action: 'loading', label: t('demo.loading') },
    { action: 'empty', label: t('demo.empty') },
    { action: 'model_missing', label: providerMissing ? t('demo.modelBack') : t('demo.modelMissing') },
  ];
  return (
    <div className={styles.wrap} ref={menuAnchor}>
      <button type="button" className={styles.trigger} aria-expanded={menuOpen} onClick={menuToggle}>
        {t('demo.menu')}
      </button>
      {menuOpen && (
        <div className={styles.menu} role="menu">
          {items.map((item) => (
            <button
              key={item.action}
              type="button"
              role="menuitem"
              className={styles.item}
              onClick={() => {
                menuClose();
                onAction(item.action);
              }}
            >
              {item.label}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
