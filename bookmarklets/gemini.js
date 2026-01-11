function scrapeGemini() {
    const chatHistory = document.querySelectorAll('.chat-history');
    if (!chatHistory || chatHistory.length === 0) {
        throw new Error('No element with class .chat-history found. Are you on the right page?');
    }
    console.log('Found chat history elements:', chatHistory.length);
    if (chatHistory.length === 0) {
        console.log('No chat history found.');
        return;
    }
    const chatHistoryElement = chatHistory[chatHistory.length - 1];
    const turns = [];
    for (const turn of chatHistoryElement.children) {
        const userTurn = turn.children[0].textContent;
        const geminiTurn = turn.children[1].querySelector('.model-response-text').textContent;
        turns.push({ user: userTurn, gemini: geminiTurn });
    }
    console.log('Found ' + turns.length + ' turns.');
    if (turns.length > 0) {
        console.log("%c EXTRACTION SUCCESSFUL", "color: white; background: green; padding: 3px; font-weight: bold;");
    }
    return turns;
}

function downloadGeminiFile(formattedText) {
    const blob = new Blob([formattedText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'transcript.txt';
    a.click();
    URL.revokeObjectURL(url);
}

function downloadGemini() {
    const turns = scrapeGemini();
    const formattedText = turns.map(entry => {
        return `User: ${entry.user.trim()}\n\nGemini: ${entry.gemini.trim()}`;
    }).join('\n\n--------------------------------------------------\n\n');
    downloadGeminiFile(formattedText);
}

downloadGemini();