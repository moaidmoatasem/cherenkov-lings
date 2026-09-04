import React from 'react';

/**
 * A small block-level Markdown renderer for drill theory and hints.
 *
 * Deliberately not a library and deliberately not `dangerouslySetInnerHTML`:
 * theory.md is repository content, but it reaches the browser over HTTP and
 * rendering it as HTML would make any future writable path an XSS vector.
 * Everything below emits React text nodes.
 *
 * Supported: ATX headings, fenced code blocks, unordered and ordered list
 * items, blockquotes, and paragraphs, with inline `code`, **bold** and
 * *italic* inside them. Anything else renders as its own text, which is the
 * correct failure mode for a document you want to be able to read.
 */

/**
 * Inline spans. Backticks bind tightest and suppress emphasis inside them, so
 * a snippet like `a * b` is not silently italicised.
 */
const INLINE = /(`[^`]+`|\*\*[^*]+\*\*|(?<![*\w])\*[^*\n]+\*(?!\w))/g;

const inline = (text: string): React.ReactNode[] =>
  text.split(INLINE).filter(Boolean).map((part, i) => {
    if (part.startsWith('`') && part.endsWith('`') && part.length > 2) {
      return (
        <code key={i} className="l-md-inline-code">
          {part.slice(1, -1)}
        </code>
      );
    }
    if (part.startsWith('**') && part.endsWith('**') && part.length > 4) {
      return <strong key={i}>{part.slice(2, -2)}</strong>;
    }
    if (part.startsWith('*') && part.endsWith('*') && part.length > 2) {
      return <em key={i}>{part.slice(1, -1)}</em>;
    }
    return <React.Fragment key={i}>{part}</React.Fragment>;
  });

type Block =
  | { kind: 'heading'; level: number; text: string }
  | { kind: 'code'; text: string }
  | { kind: 'list'; ordered: boolean; items: string[]; start: number }
  | { kind: 'quote'; text: string }
  | { kind: 'para'; text: string };

const parse = (source: string): Block[] => {
  const blocks: Block[] = [];
  const lines = source.replace(/\r\n/g, '\n').split('\n');
  let i = 0;

  const flushParagraph = (buffer: string[]) => {
    if (buffer.length) blocks.push({ kind: 'para', text: buffer.join(' ').trim() });
    buffer.length = 0;
  };

  const paragraph: string[] = [];

  while (i < lines.length) {
    const line = lines[i];

    if (line.startsWith('```')) {
      flushParagraph(paragraph);
      const body: string[] = [];
      i += 1;
      while (i < lines.length && !lines[i].startsWith('```')) {
        body.push(lines[i]);
        i += 1;
      }
      i += 1; // closing fence
      blocks.push({ kind: 'code', text: body.join('\n') });
      continue;
    }

    const heading = /^(#{1,6})\s+(.*)$/.exec(line);
    if (heading) {
      flushParagraph(paragraph);
      blocks.push({ kind: 'heading', level: heading[1].length, text: heading[2].trim() });
      i += 1;
      continue;
    }

    const bullet = /^\s*[-*+]\s+(.*)$/.exec(line);
    const numbered = /^\s*(\d+)[.)]\s+(.*)$/.exec(line);
    if (bullet || numbered) {
      flushParagraph(paragraph);
      const ordered = Boolean(numbered);
      // Where the run starts in the source. A numbered run interrupted by a
      // paragraph or a code block becomes a second <ol>, and without this every
      // such list restarted at 1 -- theory documents that step 1., 2., 3. all
      // rendered as "1.".
      const start = numbered ? Number.parseInt(numbered[1], 10) || 1 : 1;
      const items: string[] = [];
      while (i < lines.length) {
        const next = ordered
          ? /^\s*\d+[.)]\s+(.*)$/.exec(lines[i])
          : /^\s*[-*+]\s+(.*)$/.exec(lines[i]);
        if (!next) break;
        items.push(next[1].trim());
        i += 1;
      }
      blocks.push({ kind: 'list', ordered, items, start });
      continue;
    }

    if (line.startsWith('>')) {
      flushParagraph(paragraph);
      blocks.push({ kind: 'quote', text: line.replace(/^>\s?/, '').trim() });
      i += 1;
      continue;
    }

    if (line.trim() === '') {
      flushParagraph(paragraph);
      i += 1;
      continue;
    }

    paragraph.push(line.trim());
    i += 1;
  }

  flushParagraph(paragraph);
  return blocks;
};

export const Markdown: React.FC<{ source: string }> = ({ source }) => (
  <div className="l-md">
    {parse(source).map((block, index) => {
      switch (block.kind) {
        case 'heading': {
          const Tag = `h${Math.min(block.level + 1, 6)}` as 'h2';
          return <Tag key={index}>{inline(block.text)}</Tag>;
        }
        case 'code':
          return (
            <pre key={index} className="l-md-code">
              <code>{block.text}</code>
            </pre>
          );
        case 'list':
          return block.ordered ? (
            <ol key={index} start={block.start}>
              {block.items.map((item, j) => (
                <li key={j}>{inline(item)}</li>
              ))}
            </ol>
          ) : (
            <ul key={index}>
              {block.items.map((item, j) => (
                <li key={j}>{inline(item)}</li>
              ))}
            </ul>
          );
        case 'quote':
          return (
            <blockquote key={index} className="l-md-quote">
              {inline(block.text)}
            </blockquote>
          );
        default:
          return <p key={index}>{inline(block.text)}</p>;
      }
    })}
  </div>
);
