import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { MockWorkspaceGateway, type MockTiming } from '../mock/mockGateway';
import { App } from './App';

const FAST: MockTiming = {
  loadMs: 0,
  slowLoadMs: 0,
  thinkingMs: 0,
  tickMs: 0,
  charsPerTick: 1000,
  stepMs: 2,
};

async function openPlan(timing: MockTiming = FAST) {
  const gateway = new MockWorkspaceGateway(timing);
  const user = userEvent.setup();
  render(<App gateway={gateway} demo={gateway} />);
  await screen.findByRole('heading', { name: 'Thumbnail pentru joc' });
  await user.click(screen.getByRole('button', { name: /De ce checkpointul e pe server/ }));
  await user.click(await screen.findByRole('button', { name: 'Pregătește un plan' }));
  const card = await screen.findByRole('region', { name: /Plan v1/ });
  await user.click(within(card).getByRole('button', { name: 'Deschide planul' }));
  await screen.findByRole('heading', { name: /Plan v1: Obby de test/ });
  return { gateway, user };
}

describe('Plan → aprobare → execuție → rezultate (gateway demo)', () => {
  it('fără aprobare nu există execuție; tabelul și testele sunt afișate', async () => {
    const { user } = await openPlan();
    expect(screen.getAllByRole('row')).toHaveLength(3);
    expect(screen.getByText('Scrie traseul și checkpointul pe server')).toBeInTheDocument();
    expect(screen.getAllByText('Studio · neexecutat în M1')).toHaveLength(2);
    expect(screen.queryByRole('button', { name: 'Pornește execuția' })).not.toBeInTheDocument();
    await user.click(screen.getByRole('tab', { name: 'Execuție' }));
    expect(screen.getByText('Execuția pornește după aprobare')).toBeInTheDocument();
  });

  it('aprobarea arată semnătura și amprenta; revocarea readuce planul în așteptare', async () => {
    const { user } = await openPlan();
    await user.click(screen.getByRole('button', { name: 'Aprobă planul v1' }));
    expect(await screen.findByText(/Aprobat de DEMO\\utilizator/)).toBeInTheDocument();
    expect(screen.getByText(/amprenta planului sha256 [0-9a-f]{4}…[0-9a-f]{4}/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Revocă aprobarea' }));
    expect(await screen.findByRole('button', { name: 'Aprobă planul v1' })).toBeInTheDocument();
    expect(screen.getByText(/Aprobare revocată la/)).toBeInTheDocument();
  });

  it('execuția rulează pașii reali ai simulării și rezultatele sunt oneste', async () => {
    const { user } = await openPlan();
    await user.click(screen.getByRole('button', { name: 'Aprobă planul v1' }));
    await user.click(await screen.findByRole('button', { name: 'Pornește execuția' }));
    await user.click(screen.getByRole('tab', { name: 'Execuție' }));
    expect(await screen.findByText('Scris src/server/Course.server.luau · 8 linii')).toBeInTheDocument();
    await screen.findByRole('button', { name: 'Vezi rezultatele' }, { timeout: 3000 });
    expect(screen.getByText('100%')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Vezi rezultatele' }));
    const verdict = await screen.findByRole('status');
    expect(within(verdict).getByText('Necesită verificare')).toBeInTheDocument();
    expect(screen.getAllByLabelText('passed')).toHaveLength(3);
    expect(screen.getAllByLabelText('not_run')).toHaveLength(2);
    expect(screen.queryByText('Finalizat')).not.toBeInTheDocument();
    expect(screen.getByRole('tab', { name: /Course\.server\.luau/ })).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await user.click(screen.getByRole('tab', { name: /static-checks\.md/ }));
    expect(screen.getByText('reports/static-checks.md', { selector: 'div' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Publică în Roblox…' })).toBeDisabled();
  });

  it('pauza oprește pașii noi, „Continuă” îi reia, „Oprește” încheie rularea', async () => {
    const { user } = await openPlan({ ...FAST, stepMs: 60 });
    await user.click(screen.getByRole('button', { name: 'Aprobă planul v1' }));
    await user.click(await screen.findByRole('button', { name: 'Pornește execuția' }));
    await user.click(screen.getByRole('tab', { name: 'Execuție' }));
    await user.click(await screen.findByRole('button', { name: 'Pauză' }));
    expect(
      (await screen.findAllByText('În pauză. Niciun pas nou nu pornește.', {}, { timeout: 2000 })).length,
    ).toBeGreaterThan(0);
    await user.click(screen.getByRole('button', { name: 'Continuă' }));
    expect((await screen.findAllByText('Rulare reluată')).length).toBeGreaterThan(0);
    await user.click(screen.getByRole('button', { name: 'Oprește' }));
    expect((await screen.findAllByText(/Rulare oprită de utilizator/)).length).toBeGreaterThan(0);
    await user.click(screen.getByRole('button', { name: 'Vezi rezultatele' }));
    const verdict = await screen.findByRole('status');
    expect(within(verdict).getByText('Oprit')).toBeInTheDocument();
  });

  it('după o repornire, fereastra de recuperare cere verificare înainte de a continua', async () => {
    const { user } = await openPlan({ ...FAST, stepMs: 80 });
    await user.click(screen.getByRole('button', { name: 'Aprobă planul v1' }));
    await user.click(await screen.findByRole('button', { name: 'Pornește execuția' }));
    await user.click(screen.getByRole('button', { name: '← Conversație' }));
    await user.click(screen.getByRole('button', { name: 'Demo: stări' }));
    await user.click(screen.getByRole('menuitem', { name: 'Repornire în timpul rulării' }));
    const dialog = await screen.findByRole('dialog', { name: 'Am reluat de unde rămăsesem' });
    await user.click(within(dialog).getByRole('button', { name: 'Verifică și continuă' }));
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
    expect((await screen.findAllByText(/nu s-a terminat; se reia în siguranță/)).length).toBeGreaterThan(0);
    await screen.findByRole('button', { name: 'Vezi rezultatele' }, { timeout: 4000 });
  });

  it('„Plan și sarcini” din bara laterală deschide planul; fără plan arată starea goală', async () => {
    const gateway = new MockWorkspaceGateway(FAST);
    const user = userEvent.setup();
    render(<App gateway={gateway} demo={gateway} />);
    await screen.findByRole('heading', { name: 'Thumbnail pentru joc' });
    await user.click(screen.getByRole('button', { name: 'Plan și sarcini' }));
    expect(await screen.findByText('Niciun plan încă')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Înapoi la conversație' }));
    expect(await screen.findByRole('heading', { name: 'Thumbnail pentru joc' })).toBeInTheDocument();
  });
});
