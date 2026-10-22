# Despre exemple

Fișierele JSON descriu contractele propuse ale aplicației. Sunt valide sintactic, dar nu există încă un loader sau runtime care să le aplice. M0 trebuie să creeze schemele și validarea semantică.

`example_only: true` și placeholder-ele împiedică folosirea lor ca configurație de producție. Wizardul generează o configurație reală cu `example_only: false`, rădăcini canonice și model local verificat. Nu înlocui cheile cu secrete: `credential_ref` este o referință logică de implementat, nu un secret sau API Windows existent.

Lista de tooluri definește vocabularul intern propus, nu numele toolurilor Roblox MCP. Adaptoarele fac maparea după discovery. Agenții au capabilități de propunere; `project.apply_validated_patch` și alte efecte aparțin executorului, după policy gate. Permisiunea efectivă este intersecția politicii globale, capabilităților rolului/executorului și planului aprobat.

Configul implicit permite cercetare numai pe două domenii tehnice. Google, YouTube, Reddit și alte surse se activează prin configurare verificată și plan explicit, după implementarea adaptorului potrivit. Furnizorii API, push, upload și publish rămân dezactivați până la configurare și aprobare.

Hash-ul planului, snapshotul și approval sunt generate de aplicație, nu completate de model. Nu seta manual `approved` pentru a ocoli fluxul UI.
