import { useLanguage } from "../i18n/LanguageContext";

interface Props {
  /** Volle Breite, passend in die Sidebar-Buttonliste (statt kleiner Pille) */
  full?: boolean;
}

export default function LanguageToggle({ full }: Props) {
  const { lang, setLang } = useLanguage();

  return (
    <div className={`language-toggle ${full ? "language-toggle-full" : ""}`}>
      <button className={lang === "de" ? "active" : ""} onClick={() => setLang("de")}>
        Deutsch
      </button>
      <button className={lang === "en" ? "active" : ""} onClick={() => setLang("en")}>
        English
      </button>
    </div>
  );
}
