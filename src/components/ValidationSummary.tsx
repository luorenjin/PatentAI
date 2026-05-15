import type { ValidationIssue } from "../types/patentSession";

interface ValidationSummaryProps {
  issues: ValidationIssue[];
}

export function ValidationSummary({ issues }: ValidationSummaryProps) {
  if (issues.length === 0) {
    return (
      <div className="validationPanel validationPanel-ok">
        <div className="validationHeader">
          <span className="validationBadge validationBadge-ok">校验通过</span>
          <span className="validationCount">8 章节 + 2 专项输出结构完整</span>
        </div>
        <p className="validationDescription">
          当前版本已经满足 demo 级输出格式要求，可继续补充内容或直接下载当前版本。
        </p>
      </div>
    );
  }

  return (
    <div className="validationPanel validationPanel-warning">
      <div className="validationHeader">
        <span className="validationBadge validationBadge-warning">待修正</span>
        <span className="validationCount">{issues.length} 项格式提示</span>
      </div>
      <ul className="validationList">
        {issues.map((issue, index) => (
          <li
            key={`${issue.code}-${issue.sectionId ?? index}`}
            className={
              issue.level === "error"
                ? "validationItem validationItem-error"
                : "validationItem validationItem-warning"
            }
          >
            {issue.sectionId ? `${issue.sectionId}：${issue.message}` : issue.message}
          </li>
        ))}
      </ul>
    </div>
  );
}