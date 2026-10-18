import { Nav } from "./components/nav";
import { Hero } from "./components/hero";
import { Showcase } from "./components/showcase";
import { Bento } from "./components/bento";
import { Privacy } from "./components/privacy";
import { Install } from "./components/install";
import { Faq } from "./components/faq";
import { Footer } from "./components/footer";

export function App() {
  return (
    <>
      <Nav />
      <main>
        <Hero />
        <Showcase />
        <Bento />
        <Privacy />
        <Install />
        <Faq />
      </main>
      <Footer />
    </>
  );
}
