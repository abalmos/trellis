import subprocess

# Keep the previously audited packet transform exact, then apply narrow root
# fixes as separate scratch transforms. None of these scripts land on rs.
BASE_TRANSFORM_BLOB = "4fc833179fa27288e08b87676a794ffbfa7db890"
base = subprocess.check_output(["git", "cat-file", "blob", BASE_TRANSFORM_BLOB], text=True)
exec(compile(base, f"<blob {BASE_TRANSFORM_BLOB}>", "exec"), {})

for script in [
    "agent_wasm_boundary_v3_normalization_fix.py",
    "agent_wasm_boundary_v3_eager_digest_fix.py",
    "agent_wasm_boundary_v3_client_identity_fix.py",
    "agent_wasm_boundary_v3_client_identity_cleanup.py",
    "agent_wasm_boundary_v3_remaining_digest_fix.py",
]:
    source = subprocess.check_output(
        ["git", "show", f"FETCH_HEAD:scripts/{script}"],
        text=True,
    )
    exec(compile(source, script, "exec"), {})
