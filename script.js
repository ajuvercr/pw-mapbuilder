async function write(content, name) {
    var a = document.createElement("a");
    a.href = window.URL.createObjectURL(new Blob([content], {type: "text/plain"}));
    a.download = name;
    a.click(); 
}

export function save(content) {
    write(content, "save.json");
}

export async function load() {
        const f = document.getElementById("file2");
        f.onchange = async () => {
                f.onchange = () => { };
                console.log(f.files);
                const file = f.files[0];
                const text = await file.text();

                window.scene.finish_load(text);

        };
        f.click();
}

export function exp(content) {
    write(content, "export.json");
}

