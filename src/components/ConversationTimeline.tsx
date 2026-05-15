import type { ChatMessage, MessageIntent, MessageRole } from "../types/patentSession";

interface ConversationTimelineProps {
  messages: ChatMessage[];
}

const roleLabels: Record<MessageRole, string> = {
  system: "系统",
  assistant: "AI",
  engineer: "工程师",
};

const intentLabels: Record<MessageIntent, string> = {
  status: "状态",
  diagnosis: "诊断",
  follow_up: "追问",
  answer: "回答",
  advisory: "建议",
  error: "错误",
};

export function ConversationTimeline({
  messages,
}: ConversationTimelineProps) {
  return (
    <ol className="conversationTimeline" aria-label="会话时间线">
      {messages.map((message) => (
        <li
          key={message.id}
          className={[
            "messageCard",
            `messageRole-${message.role}`,
            `messageIntent-${message.intent}`,
          ].join(" ")}
        >
          <div className="messageMeta">
            <span className="messageBadge">{roleLabels[message.role]}</span>
            <span className="messageIntentLabel">
              {intentLabels[message.intent]}
            </span>
          </div>
          <p className="messageContent">{message.content}</p>
        </li>
      ))}
    </ol>
  );
}