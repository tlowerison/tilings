init();

async function init() {
    if (typeof process == "object") {
        // We run in the npm/webpack environment.
        const [{main}] = await Promise.all([import("./index.js")]);
        main();
    } else {
        const [{main}] = await Promise.all([import("./index.js")]);
        await init();
        main();
    }
}
