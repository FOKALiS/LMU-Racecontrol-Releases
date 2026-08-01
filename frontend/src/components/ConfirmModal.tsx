interface Props {
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
  danger?: boolean;
}

export default function ConfirmModal({
  title,
  message,
  confirmLabel,
  cancelLabel,
  onConfirm,
  onCancel,
  danger,
}: Props) {
  return (
    <div className="modal-backdrop" onClick={onCancel}>
      <div className="modal confirm-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h1>{title}</h1>
          <button className="modal-close" onClick={onCancel}>✕</button>
        </div>
        <div className="modal-divider" />
        <p className="confirm-message">{message}</p>
        <div className="modal-actions">
          <button className="btn-outline" onClick={onCancel}>
            {cancelLabel}
          </button>
          <button
            className={danger ? "btn-danger" : "btn-solid"}
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}