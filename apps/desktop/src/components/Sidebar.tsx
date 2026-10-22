import { ChevronDown, ChevronsLeft, ChevronsRight, Plus } from 'lucide-react';

import { usePopover } from '../app/usePopover';
import type { ConversationScope, ConversationSummary, ProjectInfo, RecentProject } from '../domain/types';
import { t } from '../i18n';
import styles from './Sidebar.module.css';

interface SidebarProps {
  collapsed: boolean;
  project: ProjectInfo;
  recentProjects: RecentProject[];
  conversations: ConversationSummary[];
  activeId: string | null;
  query: string;
  hits: string[] | null;
  onToggle: () => void;
  onSearch: (query: string) => void;
  onOpen: (id: string) => void;
  onCreate: (scope: ConversationScope) => void;
}

const NAV_ITEMS = ['hosting', 'plan', 'changes', 'memory', 'computers', 'models'] as const;

function ConversationRow({
  conversation,
  active,
  onOpen,
}: {
  conversation: ConversationSummary;
  active: boolean;
  onOpen: (id: string) => void;
}) {
  const shared = conversation.visibility === 'shared';
  return (
    <button
      type="button"
      className={`${styles.conv} ${active ? styles.convActive : ''}`}
      aria-current={active ? 'true' : undefined}
      onClick={() => onOpen(conversation.id)}
    >
      <span className={styles.convTitle}>{conversation.title}</span>
      <span className={styles.convMeta}>
        <span className={shared && conversation.scope === 'project' ? styles.shared : undefined}>
          {shared ? t('sidebar.shared') : t('sidebar.private')}
        </span>
        <span>·</span>
        <span>{conversation.lastActivity}</span>
      </span>
    </button>
  );
}

export function Sidebar(props: SidebarProps) {
  const { collapsed, project, conversations, activeId, hits } = props;
  const {
    open: switcherOpen,
    toggle: switcherToggle,
    close: switcherClose,
    anchor: switcherAnchor,
  } = usePopover();

  if (collapsed) {
    return (
      <nav className={`${styles.sidebar} ${styles.collapsed}`} aria-label={t('sidebar.conversations')}>
        <button type="button" className={styles.expand} onClick={props.onToggle} title={t('sidebar.expand')}>
          <ChevronsRight size={16} aria-label={t('sidebar.expand')} />
        </button>
        <span className={styles.projectBadgeLarge} title={project.name}>
          {project.initials}
        </span>
        <button
          type="button"
          className={styles.newCollapsed}
          title={t('sidebar.newConversationTitle')}
          onClick={() => props.onCreate('project')}
        >
          <Plus size={18} aria-label={t('sidebar.newConversationTitle')} />
        </button>
      </nav>
    );
  }

  const visible = (c: ConversationSummary) => hits === null || hits.includes(c.id);
  const projectConvs = conversations.filter((c) => c.scope === 'project' && visible(c));
  const generalConvs = conversations.filter((c) => c.scope === 'general' && visible(c));

  return (
    <nav className={styles.sidebar} aria-label={t('sidebar.conversations')}>
      <div className={styles.top}>
        <div className={styles.switcherRow} ref={switcherAnchor}>
          <button
            type="button"
            className={styles.switcher}
            aria-expanded={switcherOpen}
            onClick={switcherToggle}
          >
            <span className={styles.projectBadge}>{project.initials}</span>
            <span className={styles.projectText}>
              <span className={styles.projectName}>{project.name}</span>
              <span className={styles.projectSub}>{project.subtitle}</span>
            </span>
            <ChevronDown size={14} className={styles.caret} aria-hidden="true" />
          </button>
          <button
            type="button"
            className={styles.collapse}
            onClick={props.onToggle}
            title={t('sidebar.collapse')}
            aria-label={t('sidebar.collapse')}
          >
            <ChevronsLeft size={16} />
          </button>
          {switcherOpen && (
            <div className={styles.popover}>
              <span className={styles.sectionLabel}>{t('sidebar.recentProjects')}</span>
              {props.recentProjects.map((p) => (
                <button
                  key={p.name}
                  type="button"
                  className={`${styles.recent} ${p.current ? styles.recentCurrent : ''}`}
                  disabled={!p.current}
                  title={p.current ? undefined : t('sidebar.nextStage')}
                  onClick={switcherClose}
                >
                  <span className={styles.recentName}>{p.name}</span>
                  <span className={styles.recentConn}>{p.connections}</span>
                </button>
              ))}
              <div className={styles.divider} />
              {(['newGame', 'continueGame', 'host', 'join'] as const).map((key) => (
                <button
                  key={key}
                  type="button"
                  className={`${styles.action} ${key === 'newGame' ? styles.actionAccent : ''}`}
                  disabled
                  title={t('sidebar.nextStage')}
                >
                  {t(`sidebar.${key}`)}
                </button>
              ))}
            </div>
          )}
        </div>
        <input
          className={styles.search}
          type="search"
          value={props.query}
          placeholder={t('sidebar.search')}
          aria-label={t('sidebar.search')}
          onChange={(e) => props.onSearch(e.target.value)}
        />
      </div>

      <div className={styles.lists}>
        <section className={styles.group}>
          <div className={styles.groupHead}>
            <span className={styles.sectionLabel}>{t('sidebar.conversations')}</span>
            <button type="button" className={styles.addLink} onClick={() => props.onCreate('project')}>
              {t('sidebar.newConversation')}
            </button>
          </div>
          {projectConvs.map((c) => (
            <ConversationRow key={c.id} conversation={c} active={c.id === activeId} onOpen={props.onOpen} />
          ))}
          {hits !== null && projectConvs.length === 0 && (
            <span className={styles.empty}>{t('sidebar.noResults')}</span>
          )}
        </section>
        <section className={styles.group}>
          <div className={styles.groupHead}>
            <span className={styles.sectionLabel}>{t('sidebar.noProject')}</span>
            <button type="button" className={styles.addLink} onClick={() => props.onCreate('general')}>
              {t('sidebar.newGeneral')}
            </button>
          </div>
          {generalConvs.map((c) => (
            <ConversationRow key={c.id} conversation={c} active={c.id === activeId} onOpen={props.onOpen} />
          ))}
        </section>
      </div>

      <div className={styles.nav}>
        {NAV_ITEMS.map((key) => (
          <button key={key} type="button" className={styles.navItem} disabled title={t('sidebar.nextStage')}>
            <span className={styles.navLabel}>{t(`sidebar.nav.${key}`)}</span>
            {key === 'hosting' && <span className={styles.navDot} />}
          </button>
        ))}
      </div>
    </nav>
  );
}
