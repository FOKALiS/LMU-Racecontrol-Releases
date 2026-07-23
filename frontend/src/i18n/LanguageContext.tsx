import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { translations, Lang, TranslationKey } from "./translations";

interface LanguageContextValue {
  lang: Lang;
  setLang: (l: Lang) => void;
  t: (key: TranslationKey, vars?: Record<string, string | number>) => string;
}

const LanguageContext = createContext<LanguageContextValue | null>(null);
const STORAGE_KEY = "lmu-rc-lang";

function detectInitialLang(): Lang {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "de" || stored === "en") return stored;
  // Automatische Erkennung nach Betriebssystem-/Browser-Spracheinstellung.
  const navLang = navigator.language || "en";
  return navLang.toLowerCase().startsWith("de") ? "de" : "en";
}

export function LanguageProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(detectInitialLang);

  function setLang(l: Lang) {
    setLangState(l);
    localStorage.setItem(STORAGE_KEY, l);
  }

  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  function t(key: TranslationKey, vars?: Record<string, string | number>): string {
    let text = translations[lang][key] ?? translations.en[key] ?? key;
    if (vars) {
      for (const [k, v] of Object.entries(vars)) {
        text = text.replace(`{${k}}`, String(v));
      }
    }
    return text;
  }

  return (
    <LanguageContext.Provider value={{ lang, setLang, t }}>{children}</LanguageContext.Provider>
  );
}

export function useLanguage() {
  const ctx = useContext(LanguageContext);
  if (!ctx) throw new Error("useLanguage() muss innerhalb von <LanguageProvider> verwendet werden");
  return ctx;
}
