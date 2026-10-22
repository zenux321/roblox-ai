import { describe, expect, it } from 'vitest';

import { formatUsd, t } from '../i18n';
import { rowsFor } from '../components/Composer';
import { MockWorkspaceGateway } from './mockGateway';

const FAST = { loadMs: 0, slowLoadMs: 0, thinkingMs: 0, tickMs: 0, charsPerTick: 1000 };

describe('MockWorkspaceGateway', () => {
  it('nu permite schimbarea vizibilității unei conversații existente', async () => {
    const gw = new MockWorkspaceGateway(FAST);
    await expect(gw.setVisibility('ideas', 'shared')).rejects.toThrow();
    const created = await gw.createConversation('project');
    expect(created.visibility).toBe('private');
    await expect(gw.setVisibility(created.id, 'shared')).resolves.toMatchObject({ visibility: 'shared' });
    await gw.sendMessage(created.id, 'primul mesaj');
    await expect(gw.setVisibility(created.id, 'private')).rejects.toThrow();
  });

  it('reîncercarea numerotează încercările și păstrează istoricul', async () => {
    const gw = new MockWorkspaceGateway(FAST);
    const { message } = await gw.sendMessage('general', 'salut');
    const first = await gw.streamReply(
      { conversationId: 'general', promptMessageId: message.id },
      new AbortController().signal,
      () => {},
    );
    const second = await gw.streamReply(
      { conversationId: 'general', promptMessageId: message.id },
      new AbortController().signal,
      () => {},
    );
    expect([first.attempt, second.attempt]).toEqual([1, 2]);
    const thread = await gw.loadMessages('general');
    expect(thread.filter((m) => m.kind === 'assistant' && m.replyTo === message.id)).toHaveLength(2);
  });

  it('fără furnizor nu generează nimic', async () => {
    const gw = new MockWorkspaceGateway(FAST);
    const { message } = await gw.sendMessage('general', 'salut');
    gw.setProviderAvailable(false);
    await expect(
      gw.streamReply(
        { conversationId: 'general', promptMessageId: message.id },
        new AbortController().signal,
        () => {},
      ),
    ).rejects.toThrow('provider_missing');
  });

  it('caută fără diferențe de majuscule și diacritice păstrate', async () => {
    const gw = new MockWorkspaceGateway(FAST);
    await expect(gw.searchConversations('GHEAȚĂ')).resolves.toEqual(['ideas']);
    await expect(gw.searchConversations('checkpoint')).resolves.toContain('checkpoint');
  });
});

describe('utilitare', () => {
  it('formatează USD în stil românesc', () => {
    expect(formatUsd(0)).toBe('0,00 $');
    expect(formatUsd(0.16)).toBe('0,16 $');
  });

  it('interpolează variabilele din ro.json', () => {
    expect(t('chat.attempt', { n: 3 })).toBe('Încercarea 3');
    expect(t('composer.context', { count: 2 })).toBe('Context: 2 din vault');
  });

  it('composerul are între 2 și 8 rânduri', () => {
    expect(rowsFor('')).toBe(2);
    expect(rowsFor('a\nb\nc')).toBe(3);
    expect(rowsFor('\n'.repeat(20))).toBe(8);
  });
});
