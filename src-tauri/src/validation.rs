use crate::models::{
    DisclosureDocument, DisclosureSection, ValidationIssue, ValidationLevel,
};

const REQUIRED_SECTIONS: [(&str, &str); 10] = [
    ("title", "一、发明名称"),
    ("field", "二、技术领域"),
    ("background", "三、背景技术及现有缺陷"),
    ("purpose", "四、发明目的"),
    ("technical_solution", "五、技术方案"),
    ("benefits", "六、有益效果"),
    ("figures", "七、附图说明"),
    ("implementation", "八、具体实施方式"),
    ("layout", "🔍 专利挖掘与布局建议"),
    ("questions", "🛠️ 需工程师补齐的关键改进建议"),
];

const BENEFITS_TABLE_HEADER: &str = "| 维度 | 技术突破点 | 效果与价值体现 |";
const LAYOUT_TABLE_HEADER: &str = "| 布局方向 | 保护策略 | 建议权利要求架构 |";

pub fn validate_disclosure_document(document: &DisclosureDocument) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    validate_required_sections(document, &mut issues);
    validate_table_header(
        document,
        "benefits",
        BENEFITS_TABLE_HEADER,
        "benefits_table_format",
        "第六部分“有益效果”必须使用“维度 | 技术突破点 | 效果与价值体现”表格格式。",
        &mut issues,
    );
    validate_table_header(
        document,
        "layout",
        LAYOUT_TABLE_HEADER,
        "layout_table_format",
        "“专利挖掘与布局建议”必须使用“布局方向 | 保护策略 | 建议权利要求架构”表格格式。",
        &mut issues,
    );
    validate_questions_format(document, &mut issues);
    validate_missing_fact_markers(document, &mut issues);

    issues
}

fn validate_required_sections(document: &DisclosureDocument, issues: &mut Vec<ValidationIssue>) {
    if document.sections.len() != REQUIRED_SECTIONS.len() {
        issues.push(issue(
            "section_count_mismatch",
            format!(
                "当前输出共包含 {} 个章节/专项，预期为 {} 个。",
                document.sections.len(),
                REQUIRED_SECTIONS.len()
            ),
            None,
            ValidationLevel::Error,
        ));
    }

    for (index, (expected_id, expected_title)) in REQUIRED_SECTIONS.iter().enumerate() {
        match document.sections.get(index) {
            Some(section)
                if section.id == *expected_id && section.title == *expected_title =>
            {
                if section.content.trim().is_empty() {
                    issues.push(issue(
                        &format!("empty_section_{}", expected_id),
                        format!("{} 当前内容为空。", expected_title),
                        Some(expected_id),
                        ValidationLevel::Error,
                    ));
                }
            }
            Some(section) => {
                issues.push(issue(
                    &format!("section_order_{}", index),
                    format!(
                        "第 {} 个章节/专项应为“{}”，当前实际为“{}”。",
                        index + 1,
                        expected_title,
                        section.title
                    ),
                    Some(section.id.as_str()),
                    ValidationLevel::Error,
                ));
            }
            None => {
                issues.push(issue(
                    &format!("missing_section_{}", expected_id),
                    format!("缺少必需章节/专项“{}”。", expected_title),
                    Some(expected_id),
                    ValidationLevel::Error,
                ));
            }
        }
    }
}

fn validate_table_header(
    document: &DisclosureDocument,
    section_id: &str,
    expected_header: &str,
    code: &str,
    message: &str,
    issues: &mut Vec<ValidationIssue>,
) {
    if let Some(section) = find_section(document, section_id) {
        if !section.content.contains(expected_header) {
            issues.push(issue(
                code,
                message.to_string(),
                Some(section_id),
                ValidationLevel::Error,
            ));
        }
    }
}

fn validate_questions_format(document: &DisclosureDocument, issues: &mut Vec<ValidationIssue>) {
    if let Some(section) = find_section(document, "questions") {
        let has_q_format = section.content.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with('Q')
                && trimmed
                    .chars()
                    .nth(1)
                    .map(|character| character.is_ascii_digit())
                    .unwrap_or(false)
        });

        if !has_q_format {
            issues.push(issue(
                "questions_q_format",
                "“需工程师补齐的关键改进建议”必须使用 Q1/Q2 问答式格式。".to_string(),
                Some("questions"),
                ValidationLevel::Error,
            ));
        }
    }
}

fn validate_missing_fact_markers(
    document: &DisclosureDocument,
    issues: &mut Vec<ValidationIssue>,
) {
    for section_id in ["technical_solution", "implementation"] {
        if let Some(section) = find_section(document, section_id) {
            if section.content.contains("待补充")
                && !section.content.contains("【待工程师补充】")
            {
                issues.push(issue(
                    &format!("missing_marker_{}", section_id),
                    format!(
                        "{} 中出现缺失信息提示时，应使用“【待工程师补充】”统一标记。",
                        section.title
                    ),
                    Some(section_id),
                    ValidationLevel::Warning,
                ));
            }
        }
    }
}

fn find_section<'a>(document: &'a DisclosureDocument, section_id: &str) -> Option<&'a DisclosureSection> {
    document.sections.iter().find(|section| section.id == section_id)
}

fn issue(
    code: &str,
    message: String,
    section_id: Option<&str>,
    level: ValidationLevel,
) -> ValidationIssue {
    ValidationIssue {
        code: code.to_string(),
        message,
        section_id: section_id.map(str::to_string),
        level,
    }
}