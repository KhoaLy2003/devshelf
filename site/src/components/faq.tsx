import { CaretDown } from "@phosphor-icons/react";
import { container, sectionPad } from "../lib/classes";

const ITEMS = [
  {
    question: "Does DevShelf send my data anywhere?",
    answer:
      "No. There is no cloud, sync or telemetry. The only network request is the update check against GitHub Releases, which you can turn off in Settings > Updates.",
  },
  {
    question: "What is encrypted and what is not?",
    answer:
      "Secrets and note bodies are encrypted. Project names, tags, environment names and file paths are stored unencrypted in %APPDATA%\\app.devshelf.desktop.",
  },
  {
    question: "What if I forget the master password?",
    answer: "The vault cannot be recovered. Keep the password somewhere safe.",
  },
  {
    question: "Does unregistering a project delete my files?",
    answer: "No. It removes only the registration. Your project files are not touched.",
  },
  {
    question: "Does it run on macOS or Linux?",
    answer: "Not today. DevShelf targets Windows 10 and 11 on x64.",
  },
];

export function Faq() {
  return (
    <section id="faq" className={`${container} ${sectionPad}`}>
      <div className="mx-auto max-w-3xl">
        <h2 className="text-3xl font-semibold tracking-tight md:text-4xl">Questions</h2>
        <div className="mt-10 divide-y divide-hairline">
          {ITEMS.map((item) => (
            <details key={item.question} className="group">
              <summary className="flex cursor-pointer list-none items-center justify-between gap-6 py-5 font-semibold [&::-webkit-details-marker]:hidden">
                {item.question}
                <CaretDown
                  size={20}
                  weight="regular"
                  aria-hidden="true"
                  className="shrink-0 transition group-open:rotate-180"
                />
              </summary>
              <p className="pb-5 text-ink-muted">{item.answer}</p>
            </details>
          ))}
        </div>
      </div>
    </section>
  );
}
