// "Fehler melden" (ADR 0027): the sheet's taps. The report is put
// together by ../report.ts from the diagnosis buffer (../diag.ts) and sent
// to the account server; offline it waits in a queue that goes out when
// the network is back (and at the next start).

import { sound } from '../audio';
import { hadTrouble, previousSession, saveDiag } from '../diag';
import { de, t } from '../i18n';
import { buildReport, flushReports, queuedReports, sendReport } from '../report';
import type { ReportOverlay } from '../ui/state';
import { copyText } from './consts';
import type { Actions, AppCtx } from './context';

export class ReportFlow {
  constructor(
    private readonly ctx: AppCtx,
    /** Where the app stands, in one line (the same as the buffer notes). */
    private readonly state: () => string,
  ) {
    this.ctx.ui.reportsWaiting = queuedReports();
    const prev = previousSession();
    this.ctx.ui.reportPrevious = prev !== null && hadTrouble(prev.entries);
  }

  private get sheet(): ReportOverlay | null {
    const o = this.ctx.ui.overlay;
    return o?.kind === 'report' ? o : null;
  }

  private show(o: ReportOverlay): void {
    this.ctx.ui.overlay = o;
    this.ctx.render();
  }

  open(): void {
    this.show({ kind: 'report', phase: 'form', text: '', withSave: true, id: null, error: null, upload: null, copied: null });
  }

  /** The form as the player left it (text and box). */
  private readForm(o: ReportOverlay): ReportOverlay {
    const root = this.ctx.root;
    const text = root.querySelector<HTMLTextAreaElement>('#report-text')?.value ?? o.text;
    const box = root.querySelector<HTMLInputElement>('#report-save');
    return { ...o, text, withSave: box ? box.checked : o.withSave };
  }

  private async build(o: ReportOverlay): Promise<{ form: ReportOverlay; upload: Awaited<ReturnType<typeof buildReport>>['upload'] } | null> {
    const form = this.readForm(o);
    const building: ReportOverlay = { ...form, phase: 'building', error: null };
    this.show(building);
    this.ctx.persist();
    saveDiag();
    try {
      const built = await buildReport(form.text, form.withSave, { settings: this.ctx.ui.settings, state: this.state(), signedIn: this.ctx.ui.account.user !== null });
      if (this.ctx.ui.overlay !== building) return null;
      return { form, upload: built.upload };
    } catch (err) {
      console.warn('report not built', err);
      if (this.ctx.ui.overlay === building) this.show({ ...form, phase: 'form', error: de.report.buildFailed });
      return null;
    }
  }

  async send(): Promise<void> {
    const o = this.sheet;
    if (!o || o.phase !== 'form') return;
    const made = await this.build(o);
    if (!made) return;
    const sending: ReportOverlay = { ...made.form, phase: 'sending', upload: made.upload.report };
    this.show(sending);
    const result = await sendReport(made.upload);
    this.ctx.ui.reportsWaiting = queuedReports();
    if (this.ctx.ui.overlay !== sending) {
      if (result.ok) this.ctx.showToast(t(de.report.sentLater, { id: result.id }));
      return;
    }
    if (result.ok) {
      sound.play('clink');
      this.show({ ...sending, phase: 'sent', id: result.id });
    } else if (result.queued) {
      this.show({ ...sending, phase: 'queued' });
    } else {
      this.show({ ...sending, phase: 'form', error: t(de.report.failed, { reason: result.code }) });
    }
  }

  /** "Als Text kopieren": the report as it would go, to copy and send by hand. */
  async asText(): Promise<void> {
    const o = this.sheet;
    if (!o || o.phase === 'building' || o.phase === 'sending') return;
    if (o.phase === 'text') return;
    if (o.upload && o.phase !== 'form') {
      this.show({ ...o, phase: 'text', from: o.phase, copied: null });
      return;
    }
    const made = await this.build(o);
    if (made) this.show({ ...made.form, phase: 'text', from: 'form', upload: made.upload.report, copied: null });
  }

  copy(): void {
    const o = this.sheet;
    if (!o?.upload) return;
    const done = (ok: boolean): void => {
      if (this.ctx.ui.overlay !== o) return;
      this.show({ ...o, copied: ok });
      if (!ok) this.select();
    };
    copyText(o.upload, done, () => {
      let ok = false;
      try {
        ok = this.select() && document.execCommand('copy');
      } catch {
        ok = false;
      }
      done(ok);
    });
  }

  private select(): boolean {
    const area = this.ctx.root.querySelector<HTMLTextAreaElement>('[data-testid="report-json"]');
    if (!area) return false;
    area.focus();
    area.select();
    area.setSelectionRange(0, area.value.length);
    return true;
  }

  /** Send what waited offline; each sent one gets a short note with its number. */
  async flush(): Promise<void> {
    if (!queuedReports()) return;
    const ids = await flushReports();
    this.ctx.ui.reportsWaiting = queuedReports();
    for (const id of ids) this.ctx.showToast(t(de.report.sentLater, { id }), 'top');
  }

  actions(): Actions {
    const ui = this.ctx.ui;
    return {
      'report-open': () => {
        if (ui.busy) return;
        this.open();
      },
      'report-send': () => void this.send(),
      'report-text': () => void this.asText(),
      'report-copy': () => this.copy(),
      'report-back': () => {
        const o = this.sheet;
        if (o) this.show({ ...o, phase: o.from ?? 'form', copied: null });
      },
    };
  }
}
