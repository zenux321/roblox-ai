import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { MockWorkspaceGateway, type MockTiming } from '../mock/mockGateway';
import { App } from './App';

const FAST: MockTiming = { loadMs: 0, slowLoadMs: 0, thinkingMs: 5, tickMs: 2, charsPerTick: 40, stepMs: 2 };
const SLOW_STREAM: MockTiming = { loadMs: 0, slowLoadMs: 0, thinkingMs: 0, tickMs: 40, charsPerTick: 2 };

async function setup(timing: MockTiming = FAST) {
  const gateway = new MockWorkspaceGateway(timing);
  const user = userEvent.setup();
  render(<App gateway={gateway} demo={gateway} />);
  await screen.findByRole('heading', { name: 'Thumbnail pentru joc' });
  await screen.findByText(/Am făcut două capturi/);
  return { gateway, user };
}

function composer() {
  return screen.getByRole('textbox', { name: 'Scrie un mesaj…' });
}

describe('Spațiul de lucru (date demonstrative)', () => {
  it('afișează shell-ul v5 fără stări fictive de conexiune', async () => {
    await setup();
    expect(screen.getByText('Mod demonstrativ')).toBeInTheDocument();
    expect(screen.getByText('Lucru individual')).toBeInTheDocument();
    const titleBar = within(screen.getByRole('banner'));
    expect(titleBar.getByText('Studio')).toBeInTheDocument();
    expect(titleBar.getByText('neconfigurat')).toBeInTheDocument();
    expect(screen.queryByText(/Studio conectat/)).not.toBeInTheDocument();
    expect(document.body.textContent).not.toMatch(/Nu ai fost taxat/);
    expect(screen.getByText('Generare imagini · OpenAI Images')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Configurează OpenAI Images' })).toBeDisabled();
  });

  it('trimite cu Enter și primește răspunsul furnizorului demo', async () => {
    const { user } = await setup();
    await user.type(composer(), 'Explică-mi cum funcționează checkpointul{Enter}');
    expect(await screen.findByText('Explică-mi cum funcționează checkpointul')).toBeInTheDocument();
    expect(composer()).toHaveValue('');
    expect(await screen.findByText(/Serverul e sursa de adevăr/, {}, { timeout: 2000 })).toBeInTheDocument();
  });

  it('Shift+Enter adaugă un rând nou fără trimitere', async () => {
    const { user } = await setup();
    await user.type(composer(), 'rândul 1{Shift>}{Enter}{/Shift}rândul 2');
    expect(composer()).toHaveValue('rândul 1\nrândul 2');
    expect(composer()).toHaveAttribute('rows', '2');
    expect(screen.getByRole('button', { name: 'Trimite' })).toBeEnabled();
  });

  it('oprește răspunsul, păstrează textul parțial, iar reîncercarea nu șterge istoricul', async () => {
    const { user } = await setup(SLOW_STREAM);
    await user.type(composer(), 'O întrebare oarecare{Enter}');
    await screen.findByText(/scrie…/);
    await user.click(screen.getByRole('button', { name: 'Oprește răspunsul' }));
    const stopped = await screen.findByText('Răspuns oprit');
    expect(stopped).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Trimite' })).toBeInTheDocument();

    const stoppedArticle = stopped.closest('article');
    expect(stoppedArticle).not.toBeNull();
    await user.click(within(stoppedArticle as HTMLElement).getByRole('button', { name: 'Reîncearcă' }));
    await screen.findByText(/scrie…/);
    await user.click(screen.getByRole('button', { name: 'Oprește răspunsul' }));
    await waitFor(() => expect(screen.getAllByText('Răspuns oprit')).toHaveLength(2));
    expect(screen.getByText('Încercarea 2')).toBeInTheDocument();
    expect(screen.getByText('O întrebare oarecare')).toBeInTheDocument();
  });

  it('redenumește cu Enter și anulează cu Escape', async () => {
    const { user } = await setup();
    await user.click(screen.getByRole('button', { name: 'Redenumește' }));
    const input = screen.getByRole('textbox', { name: 'Redenumește' });
    await user.clear(input);
    await user.type(input, 'Thumbnail final{Enter}');
    expect(await screen.findByRole('heading', { name: 'Thumbnail final' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Thumbnail final/ })).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Redenumește' }));
    await user.type(screen.getByRole('textbox', { name: 'Redenumește' }), ' nu{Escape}');
    expect(screen.getByRole('heading', { name: 'Thumbnail final' })).toBeInTheDocument();
  });

  it('caută în titluri și în conținutul mesajelor', async () => {
    const { user } = await setup();
    const search = screen.getByRole('searchbox', { name: 'Caută în conversații' });
    await user.type(search, 'gheață');
    await waitFor(() =>
      expect(screen.queryByRole('button', { name: /Thumbnail pentru joc/ })).not.toBeInTheDocument(),
    );
    expect(screen.getByRole('button', { name: /Idei pentru nivelul 2/ })).toBeInTheDocument();
    await user.clear(search);
    await user.type(search, 'xyz-inexistent');
    expect(await screen.findByText('Nicio conversație găsită.')).toBeInTheDocument();
  });

  it('restrânge panoul stâng și ascunde panoul drept', async () => {
    const { user } = await setup();
    await user.click(screen.getByRole('button', { name: 'Restrânge' }));
    expect(screen.queryByRole('searchbox')).not.toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Extinde' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Ascunde panoul' }));
    expect(screen.queryByRole('tablist')).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Arată panoul' }));
    expect(screen.getByRole('tablist')).toBeInTheDocument();
  });

  it('fără model, composerul este oprit și nu se trece la un furnizor plătit', async () => {
    const { user } = await setup();
    await user.click(screen.getByRole('button', { name: 'Demo: stări' }));
    await user.click(screen.getByRole('menuitem', { name: 'Model lipsă' }));
    expect(await screen.findByText('Modelul local lipsește')).toBeInTheDocument();
    expect(composer()).toBeDisabled();
    expect(screen.getByRole('button', { name: 'Trimite' })).toBeDisabled();
    await user.click(screen.getByRole('button', { name: /Niciun model/ }));
    await user.click(screen.getByRole('button', { name: /Claude API/ }));
    expect(screen.getByText(/Nu se trece automat la un serviciu plătit/)).toBeInTheDocument();
  });

  it('copiază blocul de cod și afișează „Copiat”', async () => {
    const { user } = await setup();
    await user.click(screen.getByRole('button', { name: /De ce checkpointul e pe server/ }));
    await screen.findByText('CheckpointService.server.luau');
    const copyButtons = screen.getAllByRole('button', { name: 'Copiază' });
    await user.click(copyButtons[0] as HTMLElement);
    expect(await screen.findByRole('button', { name: 'Copiat' })).toBeInTheDocument();
    await expect(navigator.clipboard.readText()).resolves.toContain('lastCheckpoint[player] = part');
  });

  it('randează HTML din mesaje doar ca text', async () => {
    const { user } = await setup();
    const payload = '<img src=x onerror="alert(1)"><script>alert(2)</script>';
    await user.type(
      composer(),
      payload.replace(/[{[]/g, (c) => c + c),
    );
    await user.keyboard('{Enter}');
    expect(await screen.findByText(payload)).toBeInTheDocument();
    expect(document.querySelector('img')).toBeNull();
    expect(document.querySelector('script')).toBeNull();
  });

  it('o conversație nouă pornește privată și permite alegerea vizibilității înainte de primul mesaj', async () => {
    const { user } = await setup();
    await user.click(screen.getByRole('button', { name: '+ Nouă' }));
    expect(await screen.findByText('Despre ce vrei să vorbim?')).toBeInTheDocument();
    expect(screen.getByRole('radio', { name: 'Privată · doar tu' })).toHaveAttribute('aria-checked', 'true');
    await user.click(screen.getByRole('radio', { name: 'Comună · tu și Mihai' }));
    await waitFor(() =>
      expect(screen.getByRole('radio', { name: 'Comună · tu și Mihai' })).toHaveAttribute(
        'aria-checked',
        'true',
      ),
    );
    await user.click(screen.getByRole('button', { name: 'Adaugă un cronometru pe server' }));
    expect(
      await screen.findByRole('heading', { name: 'Adaugă un cronometru pe server' }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole('button', { name: 'Pregătește un plan' }, { timeout: 2000 }),
    ).toBeInTheDocument();
  });

  it('„Pregătește un plan” adaugă cardul planului și deschide ecranul Plan', async () => {
    const { user } = await setup();
    await user.click(screen.getByRole('button', { name: /De ce checkpointul e pe server/ }));
    await user.click(await screen.findByRole('button', { name: 'Pregătește un plan' }));
    const card = await screen.findByRole('region', { name: /Plan v1 · Obby de test/ });
    expect(within(card).getByText('Așteaptă aprobarea')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Pregătește un plan' })).not.toBeInTheDocument();
    await user.click(within(card).getByRole('button', { name: 'Deschide planul' }));
    expect(await screen.findByRole('heading', { name: /Plan v1: Obby de test/ })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Aprobă planul v1' })).toBeEnabled();
  });

  it('afișează scheletul la încărcarea lentă a istoricului', async () => {
    const { gateway, user } = await setup({ ...FAST, slowLoadMs: 300 });
    void gateway;
    await user.click(screen.getByRole('button', { name: 'Demo: stări' }));
    await user.click(screen.getByRole('menuitem', { name: 'Încărcare istoric (lentă)' }));
    expect(await screen.findByText('Se încarcă istoricul conversației…')).toBeInTheDocument();
    await act(async () => {
      await new Promise((r) => setTimeout(r, 400));
    });
    expect(screen.queryByText('Se încarcă istoricul conversației…')).not.toBeInTheDocument();
  });
});
