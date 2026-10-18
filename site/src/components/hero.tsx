import { motion, useReducedMotion } from "motion/react";
import { GithubLogo, WindowsLogo } from "@phosphor-icons/react";
import { EASE } from "../lib/motion";
import { REPO_URL, RELEASES_URL, shot } from "../lib/site";
import { buttonPrimary, buttonSecondary, container, frame } from "../lib/classes";

export function Hero() {
  const reduce = useReducedMotion();

  const enterText = (index: number) =>
    reduce
      ? {}
      : {
          initial: { opacity: 0, y: 16 },
          animate: { opacity: 1, y: 0 },
          transition: { duration: 0.6, ease: EASE, delay: 0.08 * index },
        };

  const enterShot = reduce
    ? {}
    : {
        initial: { opacity: 0, x: 32 },
        animate: { opacity: 1, x: 0 },
        transition: { duration: 0.7, ease: EASE },
      };

  return (
    <section
      className={`${container} grid items-center gap-12 pt-16 pb-20 lg:grid-cols-12 lg:pt-20`}
    >
      <div className="flex flex-col gap-6 lg:col-span-6">
        <motion.h1
          {...enterText(0)}
          className="text-4xl leading-[1.05] font-semibold tracking-tight md:text-5xl lg:text-5xl"
        >
          Everything about your projects, on your machine.
        </motion.h1>
        <motion.p {...enterText(1)} className="max-w-[46ch] text-lg text-ink-muted">
          A Windows desktop app for your projects, workspaces, config files, notes and encrypted
          secrets. No cloud, no sync, no telemetry.
        </motion.p>
        <motion.div {...enterText(2)} className="flex flex-wrap gap-3">
          <a href={RELEASES_URL} rel="noopener noreferrer" className={buttonPrimary}>
            <WindowsLogo size={20} weight="regular" aria-hidden="true" />
            Download for Windows
          </a>
          <a href={REPO_URL} rel="noopener noreferrer" className={buttonSecondary}>
            <GithubLogo size={20} weight="regular" aria-hidden="true" />
            View on GitHub
          </a>
        </motion.div>
      </div>
      <motion.div
        {...enterShot}
        className={`${frame} bg-surface shadow-[0_24px_80px_-28px_rgb(43_137_255/0.35)] lg:col-span-6`}
      >
        <img
          src={shot("home")}
          alt="DevShelf Home screen showing pinned projects and recent workspaces"
          width={1100}
          height={720}
          loading="eager"
          fetchPriority="high"
          className="block h-auto w-full"
        />
      </motion.div>
    </section>
  );
}
