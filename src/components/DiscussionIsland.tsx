import { useState, useRef, useEffect } from "preact/hooks";
import type { DiscussionMessage, DiscussionSource } from "../lib/types";

export default function DiscussionIsland() {
  const [messages, setMessages] = useState<DiscussionMessage[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, loading]);

  const onSubmit = async (e: Event) => {
    e.preventDefault();
    const msg = input.trim();
    if (!msg) return;

    const userMsg: DiscussionMessage = {
      role: "user",
      content: msg,
      sources: [],
    };

    const updated = [...messages, userMsg];
    setMessages(updated);
    setInput("");
    setLoading(true);

    try {
      const resp = await fetch("/api/discussion", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ messages: updated }),
      });

      if (!resp.ok) throw new Error(`HTTP ${resp.status}`);

      const reply: DiscussionMessage = await resp.json();
      setMessages([...updated, reply]);
    } catch (e) {
      setMessages([
        ...updated,
        {
          role: "assistant",
          content: `Erreur : ${e}`,
          sources: [],
        },
      ]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <div class="discussion-messages">
        {messages.map((msg, i) => (
          <div
            key={i}
            class={`discussion-message ${msg.role === "user" ? "user" : "assistant"}`}
          >
            <div class="message-role">
              {msg.role === "user" ? "Vous" : "Bilbo"}
            </div>
            {msg.role === "assistant" ? (
              <div
                class="message-content"
                dangerouslySetInnerHTML={{ __html: msg.content }}
              />
            ) : (
              <div class="message-content">{msg.content}</div>
            )}
            {msg.sources.length > 0 && (
              <div class="message-sources">
                <strong>Sources : </strong>
                {msg.sources.map((s: DiscussionSource) => (
                  <a key={s.reference} href={`/book/${s.reference}`}>
                    {s.title}
                  </a>
                ))}
              </div>
            )}
          </div>
        ))}
        {loading && (
          <div class="discussion-message assistant">
            <div class="message-role">Bilbo</div>
            <div class="message-content">Réflexion en cours...</div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>
      <form class="discussion-input" onSubmit={onSubmit}>
        <input
          type="text"
          placeholder="Posez une question sur les livres..."
          value={input}
          onInput={(e) => setInput((e.target as HTMLInputElement).value)}
        />
        <button type="submit" disabled={loading}>
          Envoyer
        </button>
      </form>
    </div>
  );
}
