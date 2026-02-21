package landing

import japgolly.scalajs.react._
import japgolly.scalajs.react.vdom.html_<^._
import org.scalajs.dom
import scala.scalajs.js

object Main {

  private val gradient =
    "linear-gradient(-225deg, #231557 0%, #44107A 29%, #FF1361 67%, #FFF800 100%)"

  private val githubUrl = "https://github.com/aathoos/aathu-lang"

  // Accent colours pulled from the gradient
  private val pink   = "#FF1361"
  private val purple = "#44107A"

  // Gradient text is handled by the .gradient-text CSS class in index.html
  // (uses background-size + animation keyframes for the shifting effect)

  private def accentBarStyle = js.Dynamic.literal(
    background = gradient,
    height     = "3px",
    width      = "100%",
  )

  // ---------------------------------------------------------------------------
  // Navbar  — full-width rect, no float, no shadow
  // ---------------------------------------------------------------------------

  private val navGlassStyle = js.Dynamic.literal(
    background        = "rgba(255, 255, 255, 0.82)",
    backdropFilter    = "blur(10px)",
    WebkitBackdropFilter = "blur(10px)",
    borderBottom      = "1px solid rgba(229, 231, 235, 0.7)",
  )

  private val Navbar = ScalaFnComponent[Unit] { _ =>
    <.nav(
      ^.className := "w-full sticky top-0 z-50",
      ^.style     := navGlassStyle,
      <.div(
        ^.className := "max-w-6xl mx-auto px-8 h-16 flex items-center justify-between",

        // Logo
        <.a(
          ^.href      := "/",
          ^.className := "gradient-text font-mono font-bold text-xl",
          "aathu",
        ),

        // Nav links
        <.div(
          ^.className := "flex items-center gap-7",
          <.a(
            ^.href      := "#",
            ^.className := "text-sm text-gray-600 hover:text-gray-900",
            "Docs",
          ),
          <.a(
            ^.href      := "#",
            ^.className := "text-sm text-gray-600 hover:text-gray-900",
            "Examples",
          ),
          <.a(
            ^.href      := "#",
            ^.className := "text-sm text-gray-600 hover:text-gray-900",
            "Changelog",
          ),
          <.a(
            ^.href      := githubUrl,
            ^.target    := "_blank",
            ^.rel       := "noopener noreferrer",
            ^.className := "text-sm font-medium text-white px-4 py-2 rounded-md",
            ^.style     := js.Dynamic.literal(background = pink),
            "GitHub ↗",
          ),
        ),
      ),
    )
  }

  // ---------------------------------------------------------------------------
  // Hero
  // ---------------------------------------------------------------------------

  private val Hero = ScalaFnComponent[Unit] { _ =>
    <.section(
      ^.className := "max-w-6xl mx-auto px-8 pt-24 pb-20",

      // Small badge
      <.div(
        ^.className := "inline-flex items-center gap-2 text-xs font-mono px-3 py-1 rounded-full mb-8 border",
        ^.style     := js.Dynamic.literal(
          color           = purple,
          borderColor     = "#e9d5ff",
          backgroundColor = "#faf5ff",
        ),
        "Written in Rust · Open Source · Early Development",
      ),

      <.h1(
        ^.className := "text-6xl font-bold tracking-tight text-gray-900 leading-tight mb-6",
        "A compiled language",
        <.br(),
        "built for automation.",
      ),
      <.p(
        ^.className := "text-lg text-gray-500 max-w-2xl leading-relaxed mb-10",
        "aathu compiles to bytecode and runs on a custom VM. " +
        "Clean syntax, static types, and a full toolchain — " +
        "formatter, linter, LSP, debugger, and package manager included.",
      ),

      // CTAs
      <.div(
        ^.className := "flex items-center gap-4",
        <.a(
          ^.href      := githubUrl,
          ^.target    := "_blank",
          ^.rel       := "noopener noreferrer",
          ^.className := "gradient-btn inline-block text-sm font-medium text-white px-5 py-2.5 rounded-md",
          "View on GitHub ↗",
        ),
        <.a(
          ^.href      := "#",
          ^.className := "inline-block text-sm font-medium text-gray-600 hover:text-gray-900 px-5 py-2.5",
          "Read the docs →",
        ),
      ),
    )
  }

  // ---------------------------------------------------------------------------
  // Compiler pipeline strip
  // ---------------------------------------------------------------------------

  private val pipelineSteps =
    List("Source", "Lexer", "Parser", "HIR", "Type Check", "MIR", "Codegen", "Bytecode", "VM")

  private val Pipeline = ScalaFnComponent[Unit] { _ =>
    <.section(
      ^.className := "border-t border-gray-100 bg-gray-50",
      <.div(
        ^.className := "max-w-6xl mx-auto px-8 py-12",
        <.p(
          ^.className := "text-xs font-mono uppercase tracking-widest text-gray-400 mb-6",
          "Compiler Pipeline",
        ),
        <.div(
          ^.className := "flex flex-wrap items-center gap-y-3",
          pipelineSteps.zipWithIndex.toVdomArray { case (step, idx) =>
            <.div(
              ^.key       := idx.toString,
              ^.className := "flex items-center",
              if (idx > 0)
                <.span(^.className := "text-gray-300 text-sm mx-2", "→")
              else
                EmptyVdom,
              <.span(
                ^.className := "text-xs font-mono px-3 py-1.5 bg-white border border-gray-200 rounded text-gray-700",
                step,
              ),
            )
          },
        ),
      ),
    )
  }

  // ---------------------------------------------------------------------------
  // Code block
  // ---------------------------------------------------------------------------

  private val codeExample =
    """|fn greet(name) {
       |  print("Hello, " + name + "!")
       |}
       |
       |fn main() {
       |  greet("world")
       |
       |  for i in 0..5 {
       |    print(i)
       |  }
       |}""".stripMargin

  private val CodeBlock = ScalaFnComponent[Unit] { _ =>
    <.section(
      ^.className := "border-t border-gray-100",
      <.div(
        ^.className := "max-w-6xl mx-auto px-8 py-20",
        <.p(
          ^.className := "text-xs font-mono uppercase tracking-widest text-gray-400 mb-6",
          "Example",
        ),
        <.div(
          ^.className := "rounded-lg overflow-hidden bg-gray-950 max-w-2xl",
          <.div(
            ^.className := "flex items-center gap-2 px-4 py-3 border-b border-gray-800",
            <.span(^.className := "w-2.5 h-2.5 rounded-full bg-gray-700"),
            <.span(^.className := "w-2.5 h-2.5 rounded-full bg-gray-700"),
            <.span(^.className := "w-2.5 h-2.5 rounded-full bg-gray-700"),
            <.span(^.className := "ml-2 text-xs text-gray-500 font-mono", "hello.aathu"),
          ),
          <.pre(
            ^.className := "px-6 py-5 text-sm font-mono text-gray-300 leading-relaxed overflow-x-auto",
            <.code(codeExample),
          ),
        ),
      ),
    )
  }

  // ---------------------------------------------------------------------------
  // Features
  // ---------------------------------------------------------------------------

  private final case class Feature(title: String, desc: String)

  private val featureList = List(
    Feature("Compiler",
      "Full pipeline — Lexer, Parser, HIR, MIR, Codegen — compiles down to bytecode."),
    Feature("Type Checker",
      "Static type inference with precise source spans and actionable diagnostics."),
    Feature("Bytecode VM",
      "Custom register-based VM with a minimal, fast execution runtime."),
    Feature("Formatter",
      "Opinionated code formatter (aathu-fmt) — one style, zero configuration."),
    Feature("LSP Server",
      "Full language server (aathu-lsp) for editor integration and autocomplete."),
    Feature("Package Manager",
      "Built-in package manager (aathu-pkg) for managing project dependencies."),
  )

  private val Features = ScalaFnComponent[Unit] { _ =>
    <.section(
      ^.className := "border-t border-gray-100",
      <.div(
        ^.className := "max-w-6xl mx-auto px-8 py-20",
        <.p(
          ^.className := "text-xs font-mono uppercase tracking-widest text-gray-400 mb-6",
          "What's included",
        ),
        <.div(
          ^.className := "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-x-10 gap-y-10",
          featureList.zipWithIndex.toVdomArray { case (f, idx) =>
            <.div(
              ^.key       := idx.toString,
              ^.className := "border-t-2 pt-5",
              ^.style     := js.Dynamic.literal(borderColor = pink),
              <.h3(
                ^.className := "font-semibold text-gray-900 text-sm mb-2",
                f.title,
              ),
              <.p(
                ^.className := "text-sm text-gray-500 leading-relaxed",
                f.desc,
              ),
            )
          },
        ),
      ),
    )
  }

  // ---------------------------------------------------------------------------
  // Footer
  // ---------------------------------------------------------------------------

  private val Footer = ScalaFnComponent[Unit] { _ =>
    <.footer(
      ^.className := "border-t border-gray-200 bg-gray-50",
      <.div(
        ^.className := "max-w-6xl mx-auto px-8 py-10 flex flex-col sm:flex-row items-start justify-between gap-8",

        // Left — brand
        <.div(
          <.span(
            ^.className := "gradient-text font-mono font-bold text-base mb-1 block",
            "aathu",
          ),
          <.p(
            ^.className := "text-xs text-gray-400 mt-1",
            "A compiled language built for automation.",
          ),
        ),

        // Right — links
        <.div(
          ^.className := "flex gap-8 text-sm text-gray-500",
          <.div(
            ^.className := "flex flex-col gap-2",
            <.a(^.href := "#",       ^.className := "hover:text-gray-900", "Docs"),
            <.a(^.href := "#",       ^.className := "hover:text-gray-900", "Examples"),
            <.a(^.href := "#",       ^.className := "hover:text-gray-900", "Changelog"),
          ),
          <.div(
            ^.className := "flex flex-col gap-2",
            <.a(
              ^.href      := githubUrl,
              ^.target    := "_blank",
              ^.rel       := "noopener noreferrer",
              ^.className := "hover:text-gray-900",
              "GitHub",
            ),
            <.span(^.className := "text-gray-400", "MIT License"),
          ),
        ),
      ),
    )
  }

  // ---------------------------------------------------------------------------
  // Root app
  // ---------------------------------------------------------------------------

  private val App = ScalaFnComponent[Unit] { _ =>
    <.div(
      ^.className := "min-h-screen bg-white text-gray-900 antialiased",
      <.div(^.className := "gradient-bar"),
      Navbar(),
      Hero(),
      Pipeline(),
      CodeBlock(),
      Features(),
      Footer(),
    )
  }

  def main(args: Array[String]): Unit = {
    val container = dom.document.getElementById("root")
    App().renderIntoDOM(container)
  }
}
