function scrape() {
    const turns = document.querySelectorAll('[data-scope-id="turn"]');
    if (!turns || turns.length === 0) {
        throw new Error('No conversation turns found. Are you on the right page?');
    }
    console.log('Found conversation turns:', turns.length);

    const conversation = [];

    turns.forEach((turn) => {
        const userQueryEl = turn.querySelector('[role="heading"][aria-level="2"]');
        if (!userQueryEl) return;
        const userText = userQueryEl.innerText.trim();
        const responseContainer = turn.querySelector('[data-subtree="aimc"]');
        let modelText = "";
        if (responseContainer) {
            const parts = [];
            const elements = responseContainer.querySelectorAll('ol, ul, [data-sfc-cp], [role="heading"]');
            elements.forEach(el => {
                if (el.tagName === 'OL' || el.tagName === 'UL') {
                    const items = Array.from(el.querySelectorAll('li'))
                        .map(li => "• " + li.innerText.split('\n')[0].trim());
                    parts.push(items.join('\n'));
                }
                else {
                    let cleanText = el.innerText.trim();
                    if (cleanText.length < 2 || el.tagName === 'BUTTON' || el.getAttribute('role') === 'button') {
                        return;
                    }
                    if (!parts.some(p => p.includes(cleanText))) {
                        parts.push(cleanText);
                    }
                }
            });

            modelText = parts.join('\n\n');
        }

        conversation.push({
            user: userText,
            model: modelText
        });
    });

    console.log('Found ' + conversation.length + ' turns.');
    if (conversation.length > 0) {
        console.log("%c EXTRACTION SUCCESSFUL", "color: white; background: green; padding: 3px; font-weight: bold;");
    }
    return conversation;
}

function download(formattedText) {
    const blob = new Blob([formattedText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'transcript.txt';
    a.click();
    URL.revokeObjectURL(url);
}

function downloadAim() {
    const turns = scrape();
    const formattedText = turns.map(entry => {
        return `User: ${entry.user.trim()}\n\nAIM: ${entry.model.trim()}`;
    }).join('\n\n--------------------------------------------------\n\n');
    download(formattedText);
}

downloadAim();