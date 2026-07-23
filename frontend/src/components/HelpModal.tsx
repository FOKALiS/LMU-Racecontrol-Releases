import { useLanguage } from "../i18n/LanguageContext";
import { helpContentDe, helpContentEn } from "../content/helpContent";

interface Props {
  onClose: () => void;
}

export default function HelpModal({ onClose }: Props) {
  const { lang } = useLanguage();
  const sections = lang === "de" ? helpContentDe : helpContentEn;

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal help-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h1>{lang === "de" ? "Hilfe" : "Help"}</h1>
          <button className="modal-close" onClick={onClose}>
            ✕
          </button>
        </div>
        <div className="modal-divider" />

        <div className="help-content">
          {sections.map((section) => (
            <div key={section.heading}>
              <h2>{section.heading}</h2>
              {section.paragraphs.map((p, i) => (
                <p key={i}>{p}</p>
              ))}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
