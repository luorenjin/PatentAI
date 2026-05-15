import type { ReactNode } from "react";
import type {
  Annotation,
  DisclosureDocument,
  DisclosureSection,
} from "../types/patentSession";

interface PreviewPanelProps {
  disclosure: DisclosureDocument;
  agentView: boolean;
  showHighlights: boolean;
  zoom: number;
}

interface HighlightToken {
  key: string;
  annotation: Annotation;
}

export function PreviewPanel({
  disclosure,
  agentView,
  showHighlights,
  zoom,
}: PreviewPanelProps) {
  const visibleSections = disclosure.sections.filter((section) => {
    if (!agentView) {
      return true;
    }

    return disclosure.annotations.some(
      (annotation) => annotation.sectionId === section.id,
    );
  });

  return (
    <div
      className="previewDocument"
      style={{ fontSize: `${Math.max(85, Math.min(zoom, 130))}%` }}
    >
      {visibleSections.length === 0 ? (
        <div className="previewEmpty">当前代理人视图下没有可显示的高亮段落。</div>
      ) : (
        visibleSections.map((section) => {
          const sectionAnnotations = disclosure.annotations.filter(
            (annotation) => annotation.sectionId === section.id,
          );

          return (
            <section key={section.id} className="previewSection">
              <header className="previewSectionHeader">
                <h3>{section.title}</h3>
                <span className="previewSectionMeta">
                  {sectionAnnotations.length} 条标注
                </span>
              </header>
              <div className="previewSectionBody">
                {renderSectionContent(section, sectionAnnotations, showHighlights)}
              </div>
              {sectionAnnotations.length > 0 ? (
                <ul className="annotationNotes">
                  {sectionAnnotations.map((annotation) => (
                    <li key={annotation.id}>
                      <span
                        className={
                          annotation.kind === "addition"
                            ? "annotationDot annotationDot-addition"
                            : "annotationDot annotationDot-rewrite"
                        }
                      />
                      {annotation.note}
                    </li>
                  ))}
                </ul>
              ) : null}
            </section>
          );
        })
      )}
    </div>
  );
}

function renderSectionContent(
  section: DisclosureSection,
  annotations: Annotation[],
  showHighlights: boolean,
) {
  if (!showHighlights || annotations.length === 0) {
    return <p className="previewSectionText">{section.content}</p>;
  }

  let parts: Array<string | HighlightToken> = [section.content];

  annotations.forEach((annotation, annotationIndex) => {
    if (!annotation.excerpt) {
      return;
    }

    parts = parts.flatMap((part, partIndex) => {
      if (typeof part !== "string") {
        return [part];
      }

      const segments = part.split(annotation.excerpt);
      if (segments.length === 1) {
        return [part];
      }

      const nextParts: Array<string | HighlightToken> = [];
      segments.forEach((segment, segmentIndex) => {
        if (segment) {
          nextParts.push(segment);
        }

        if (segmentIndex < segments.length - 1) {
          nextParts.push({
            key: `${annotation.id}-${annotationIndex}-${partIndex}-${segmentIndex}`,
            annotation,
          });
        }
      });

      return nextParts;
    });
  });

  return (
    <p className="previewSectionText">
      {parts.map((part, index) => renderPart(part, index))}
    </p>
  );
}

function renderPart(part: string | HighlightToken, index: number): ReactNode {
  if (typeof part === "string") {
    return <span key={`text-${index}`}>{part}</span>;
  }

  return (
    <mark
      key={part.key}
      className={
        part.annotation.kind === "addition"
          ? "inlineAnnotation inlineAnnotation-addition"
          : "inlineAnnotation inlineAnnotation-rewrite"
      }
      title={part.annotation.note}
    >
      {part.annotation.excerpt}
    </mark>
  );
}