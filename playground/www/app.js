// Spween Playground App

let playground = null;
let wasm = null;

// Example scenes - embedded for simplicity
const EXAMPLES = {
    tavern: `---
id: tavern_encounter
title: The Mysterious Stranger
weight: 10
cooldown: 5
---

=== intro

A hooded figure sits alone in the corner of the tavern.
They gesture for you to approach.

* [Approach cautiously] { perception >= 10 }
  ~ perception_used = true
  -> cautious

* [Approach boldly]
  -> bold

* [Ignore them]
  ~ reputation -= 1
  -> END

=== cautious

You notice a glint of steel beneath their cloak. A weapon,
but kept hidden. They mean no immediate harm.

The stranger nods approvingly.

* [Sit down]
  ~ stranger_trust += 2
  -> conversation

* [Keep your distance]
  ~ stranger_trust += 1
  -> conversation

=== bold

You stride over confidently. The stranger seems amused
by your directness.

* [Sit down]
  ~ stranger_trust += 1
  -> conversation

=== conversation

The stranger leans close and whispers.

"I have information about the artifact you seek.
But nothing in this world is free."

* [Offer gold] { gold >= 50 }
  ~ gold -= 50
  ~ has_info = true
  -> reward

* [Offer a favor]
  ~ owes_favor = true
  ~ has_info = true
  -> reward

* [Decline and leave]
  -> END

=== reward

The stranger slides a worn map across the table.

"The tomb lies three days east, past the Darkwood.
Be warned: you are not the only one seeking it."

~ notify "quest_updated"

* [Thank them and leave]
  -> END
`,

    treasure: `---
id: treasure_hunt
title: The Hidden Treasure
weight: 10
cooldown: 5
---

=== intro

You stand before an ancient stone door covered in runes.
A puzzle mechanism awaits your input.

Your torch flickers in the dusty air.

* [Examine the runes] { intelligence >= 8 }
  ~ examined_runes = true
  -> examine

* [Try to force the door] { strength >= 12 }
  ~ strength -= 2
  -> force

* [Search for another way]
  -> search

=== examine

The runes describe a sequence: sun, moon, star.
Three symbols are carved into buttons on the door.

* [Press sun, moon, star]
  ~ solved_puzzle = true
  -> success

* [Press moon, star, sun]
  ~ trap_triggered = true
  ~ health -= 10
  -> trap

* [Press randomly]
  ~ trap_triggered = true
  ~ health -= 15
  -> trap

=== force

You throw your weight against the door.
It groans but doesn't budge.

* [Try again] { strength >= 10 }
  ~ strength -= 2
  -> force_success

* [Give up and examine the runes] { intelligence >= 6 }
  -> examine

* [Leave]
  -> END

=== force_success

With a tremendous crash, the door gives way!
Stone dust fills the air.

~ door_forced = true

* [Enter carefully]
  -> treasure

=== search

You search the walls for hidden passages.
Behind a loose stone, you find a rusty lever.

* [Pull the lever]
  ~ secret_found = true
  -> success

* [Ignore it and try the door]
  -> intro

=== trap

Poison darts shoot from the walls!
You barely dodge in time.

* [Try again more carefully]
  -> examine

* [Retreat]
  -> END

=== success

The door slides open with a grinding sound.
Ancient air rushes past you.

* [Enter]
  -> treasure

=== treasure

Inside, golden coins spill from rotted chests.
Jeweled artifacts line the walls.

You've found the legendary treasure!

~ gold += 1000
~ treasure_found = true
~ notify "achievement_unlocked"

* [Collect everything]
  ~ greedy = true
  ~ gold += 500
  -> END

* [Take only what you need]
  ~ gold += 200
  ~ karma += 5
  -> END
`,

    combat: `---
id: combat_encounter
title: The Forest Ambush
weight: 10
cooldown: 5
---

=== intro

Bandits emerge from the treeline, weapons drawn.
Their leader sneers at you.

"Your gold or your life, traveler!"

~ enemy_health = 30
~ combat_started = true

* [Draw your weapon] { has_weapon }
  -> fight

* [Try to negotiate]
  -> negotiate

* [Attempt to flee] { agility >= 10 }
  -> flee

* [Surrender]
  ~ gold = 0
  ~ surrendered = true
  -> END

=== fight

You draw your blade and take a fighting stance.
The bandits spread out to surround you.

* [Attack the leader]
  ~ damage_dealt = 15
  ~ enemy_health -= 15
  -> attack_result

* [Defend and wait]
  ~ defensive_stance = true
  -> defend

* [Use a special move] { special_moves >= 1 }
  ~ special_moves -= 1
  ~ damage_dealt = 25
  ~ enemy_health -= 25
  -> attack_result

=== attack_result

Your strike lands true!

* [Continue fighting] { enemy_health > 0 }
  -> fight

* [They're defeated] { enemy_health <= 0 }
  ~ bandits_defeated = true
  ~ gold += 25
  ~ reputation += 2
  -> victory

=== defend

You raise your guard. The bandit's attack glances off.

* [Counter-attack]
  ~ damage_dealt = 20
  ~ enemy_health -= 20
  -> attack_result

* [Continue defending]
  -> defend

=== negotiate

"Perhaps we can reach an agreement," you say calmly.

* [Offer half your gold] { gold >= 20 }
  ~ gold -= 10
  ~ negotiated = true
  -> END

* [Bluff about reinforcements] { charisma >= 12 }
  ~ bluff_success = true
  -> bluff

* [They're not interested]
  -> fight

=== bluff

"My companions are just behind me. You're outnumbered."

The bandits exchange nervous glances.
Their leader curses and signals a retreat.

~ reputation += 1

* [Let them go]
  -> END

* [Attack while they're distracted] { has_weapon }
  ~ damage_dealt = 20
  ~ enemy_health -= 20
  ~ honorable = false
  -> attack_result

=== flee

You sprint into the underbrush.
Branches whip at your face as you run.

* [Keep running]
  ~ escaped = true
  ~ stamina -= 5
  -> END

* [Circle back to ambush them] { agility >= 14 }
  ~ flanked = true
  ~ damage_dealt = 25
  ~ enemy_health -= 25
  -> attack_result

=== victory

The bandits lie defeated.
You search their camp and find some supplies.

~ combat_won = true
~ notify "combat_complete"

* [Take their gear]
  ~ looted = true
  ~ gold += 15
  -> END

* [Leave them]
  ~ merciful = true
  ~ karma += 3
  -> END
`
};

// DOM Elements
const editorEl = document.getElementById('editor');
const parseErrorEl = document.getElementById('parse-error');
const exampleSelect = document.getElementById('example-select');
const runBtn = document.getElementById('run-btn');
const restartBtn = document.getElementById('restart-btn');
const proseEl = document.getElementById('prose');
const choicesEl = document.getElementById('choices');
const variablesEl = document.getElementById('variables');
const callsEl = document.getElementById('calls');
const endedMessageEl = document.getElementById('ended-message');
const sceneTitleEl = document.getElementById('scene-title');
const passageNameEl = document.getElementById('passage-name');
const editorPanel = document.getElementById('editor-panel');
const runnerPanel = document.getElementById('runner-panel');
const tabs = document.querySelectorAll('.tab');

// Initialize
async function init() {
    try {
        // Load WASM module
        wasm = await import('./pkg/spween_playground.js');
        await wasm.default();

        playground = new wasm.Playground();

        // Set up event listeners
        setupEventListeners();

        // Load default example
        loadExample('tavern');

        console.log('Spween Playground initialized');
    } catch (e) {
        console.error('Failed to initialize:', e);
        showError('Failed to load WebAssembly module. Please refresh the page.');
    }
}

function setupEventListeners() {
    // Example selector
    exampleSelect.addEventListener('change', (e) => {
        if (e.target.value) {
            loadExample(e.target.value);
        }
    });

    // Run button
    runBtn.addEventListener('click', runScene);

    // Restart button
    restartBtn.addEventListener('click', runScene);

    // Tab switching (mobile)
    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            switchTab(tab.dataset.tab);
        });
    });

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        // Ctrl/Cmd + Enter to run
        if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            e.preventDefault();
            runScene();
        }
    });
}

function loadExample(name) {
    if (EXAMPLES[name]) {
        editorEl.value = EXAMPLES[name];
        exampleSelect.value = name;
        hideError();
    }
}

function runScene() {
    const source = editorEl.value;

    if (!source.trim()) {
        showError('Please enter a scene to run.');
        return;
    }

    try {
        // Parse the scene
        playground.parse(source);
        hideError();

        // Set some default variables for the examples
        playground.set_var('perception', '15');
        playground.set_var('gold', '100');
        playground.set_var('intelligence', '12');
        playground.set_var('strength', '10');
        playground.set_var('agility', '12');
        playground.set_var('charisma', '10');
        playground.set_var('health', '100');
        playground.set_var('stamina', '50');
        playground.set_var('special_moves', '2');
        playground.add_has('inventory', 'weapon');

        // Alias for condition
        playground.set_var('has_weapon', 'true');

        // Start the scene
        playground.start();

        // Update UI
        updateRunner();

        // Switch to runner tab on mobile
        switchTab('runner');
    } catch (e) {
        showError(e.toString());
    }
}

function updateRunner() {
    // Update title
    const title = playground.get_title();
    sceneTitleEl.textContent = title || '';

    // Update passage name
    const passageName = playground.get_passage_name();
    passageNameEl.textContent = passageName ? `// ${passageName}` : '';

    // Update prose
    const prose = playground.get_prose();
    proseEl.textContent = prose || '';

    // Update choices
    const choices = playground.get_choices();
    renderChoices(choices);

    // Update variables
    const variables = playground.get_variables();
    renderVariables(variables);

    // Update calls
    const calls = playground.get_calls();
    renderCalls(calls);

    // Show/hide ended message
    const ended = playground.is_ended();
    endedMessageEl.hidden = !ended;
    choicesEl.hidden = ended;
}

function renderChoices(choices) {
    choicesEl.innerHTML = '';

    choices.forEach((choice, i) => {
        const btn = document.createElement('button');
        btn.className = 'choice-btn';
        btn.disabled = !choice.available;
        btn.innerHTML = `<span class="choice-index">${i + 1}</span>${escapeHtml(choice.text)}`;

        btn.addEventListener('click', () => selectChoice(choice.index));

        choicesEl.appendChild(btn);
    });
}

function selectChoice(index) {
    try {
        playground.select_choice(index);
        updateRunner();
    } catch (e) {
        showError(e.toString());
    }
}

function renderVariables(variables) {
    if (variables.length === 0) {
        variablesEl.innerHTML = '<div class="empty-state">No variables set</div>';
        return;
    }

    variablesEl.innerHTML = variables
        .map(v => `<div class="variable"><span class="var-name">${escapeHtml(v.name)}</span><span class="var-value">${escapeHtml(v.value)}</span></div>`)
        .join('');
}

function renderCalls(calls) {
    if (calls.length === 0) {
        callsEl.innerHTML = '<div class="empty-state">No effect calls made</div>';
        return;
    }

    callsEl.innerHTML = calls
        .map(c => `<div class="call"><span class="call-name">${escapeHtml(c.name)}</span><span class="call-args">(${c.args.map(escapeHtml).join(', ')})</span></div>`)
        .join('');
}

function switchTab(tabName) {
    // Update tab buttons
    tabs.forEach(tab => {
        const isActive = tab.dataset.tab === tabName;
        tab.classList.toggle('active', isActive);
        tab.setAttribute('aria-selected', isActive);
    });

    // Update panels
    editorPanel.classList.toggle('active', tabName === 'editor');
    editorPanel.hidden = tabName !== 'editor';
    runnerPanel.classList.toggle('active', tabName === 'runner');
    runnerPanel.hidden = tabName !== 'runner';
}

function showError(message) {
    parseErrorEl.textContent = message;
    parseErrorEl.hidden = false;
}

function hideError() {
    parseErrorEl.hidden = true;
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Start the app
init();
