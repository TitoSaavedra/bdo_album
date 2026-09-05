<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { uploadPresetModification } from '../../../../lib/album';

  interface Props {
    presetId: string;
    onclose:  () => void;
  }

  const { presetId, onclose }: Props = $props();

  let image1     = $state<string | null>(null); // full data: URI — preview src AND invoke payload source
  let image2     = $state<string | null>(null);
  let activeSlot = $state<1 | 2>(1);
  let uploading  = $state(false);
  let queued     = $state(false);
  let error      = $state('');

  // Reads whatever image the user pastes into the currently-targeted slot —
  // no file-path dialog. Uses a data: URI (not URL.createObjectURL()'s
  // blob:) because the app's CSP only allows 'data:' in img-src, not 'blob:'.
  function handlePaste(e: ClipboardEvent) {
    const items = e.clipboardData?.items;
    if (!items) return;
    for (const item of items) {
      if (!item.type.startsWith('image/')) continue;
      e.preventDefault();
      const file = item.getAsFile();
      if (!file) continue;
      const reader = new FileReader();
      reader.onload = () => {
        const dataUrl = reader.result as string;
        if (activeSlot === 1) {
          image1 = dataUrl;
          if (!image2) activeSlot = 2;
        } else {
          image2 = dataUrl;
        }
      };
      reader.readAsDataURL(file);
      break;
    }
  }

  function stripPrefix(dataUrl: string): string {
    const i = dataUrl.indexOf(',');
    return i >= 0 ? dataUrl.slice(i + 1) : dataUrl;
  }

  async function handleSubmit() {
    if (!image1 || uploading) return;
    uploading = true;
    error = '';
    try {
      await uploadPresetModification(presetId, stripPrefix(image1), image2 ? stripPrefix(image2) : null);
      queued = true;
    } catch (e) {
      error = String(e);
    } finally {
      uploading = false;
    }
  }
</script>

<svelte:window onpaste={handlePaste} />

<div
  class="backdrop"
  role="presentation"
  onclick={onclose}
  onkeydown={(e) => e.key === 'Escape' && onclose()}
>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.stopPropagation()}
  >
    <div class="modal-header">
      <span class="modal-title">{$_('beauty.upload_modification.title')}</span>
      <button class="close-btn" onclick={onclose}>✕</button>
    </div>

    {#if queued}
      <div class="modal-body">
        <p class="queued-message">{$_('beauty.upload_modification.queued_message')}</p>
      </div>
      <div class="modal-footer">
        <button class="submit-btn" onclick={onclose}>{$_('beauty.upload_modification.close')}</button>
      </div>
    {:else}
      <div class="modal-body">
        <div class="slots">
          <div
            class="slot"
            class:active={activeSlot === 1}
            class:filled={!!image1}
            role="button"
            tabindex="0"
            onclick={() => (activeSlot = 1)}
            onkeydown={(e) => e.key === 'Enter' && (activeSlot = 1)}
          >
            {#if image1}
              <img src={image1} alt="" />
              <button class="slot-clear" onclick={(e) => { e.stopPropagation(); image1 = null; }}>✕</button>
            {:else}
              <span class="slot-hint">{$_('beauty.upload_modification.slot_1_required')}</span>
            {/if}
          </div>
          <div
            class="slot"
            class:active={activeSlot === 2}
            class:filled={!!image2}
            role="button"
            tabindex="0"
            onclick={() => (activeSlot = 2)}
            onkeydown={(e) => e.key === 'Enter' && (activeSlot = 2)}
          >
            {#if image2}
              <img src={image2} alt="" />
              <button class="slot-clear" onclick={(e) => { e.stopPropagation(); image2 = null; }}>✕</button>
            {:else}
              <span class="slot-hint">{$_('beauty.upload_modification.slot_2_optional')}</span>
            {/if}
          </div>
        </div>
        <p class="paste-hint">{$_('beauty.upload_modification.paste_hint')}</p>
        {#if error}
          <div class="error">{error}</div>
        {/if}
      </div>

      <div class="modal-footer">
        <button class="cancel-btn" onclick={onclose}>{$_('beauty.upload_modification.cancel')}</button>
        <button class="submit-btn" onclick={handleSubmit} disabled={!image1 || uploading}>
          {uploading ? $_('beauty.upload_modification.uploading') : $_('beauty.upload_modification.submit')}
        </button>
      </div>
    {/if}
  </div>
</div>

<style lang="scss">
  @use './UploadModificationModal.scss';
</style>
