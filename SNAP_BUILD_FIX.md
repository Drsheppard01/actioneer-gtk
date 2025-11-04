# Snap Build Fix Documentation

## Problem Statement
The snap-ci.yml workflow was failing with the following error:
```
The system library `glib-2.0` required by crate `glib-sys` was not found.
error: failed to run custom build command for `glib-sys v0.21.2`
```

## Root Cause Analysis
The workflow was attempting to manually cross-compile Rust code before running snapcraft:
1. It installed system packages and cross-compilation toolchains manually  
2. It ran `cargo build --release --target <architecture>` manually
3. Then it tried to run snapcraft with `--destructive-mode`

This approach was problematic because:
- The manual Rust build happened outside snapcraft's controlled environment
- pkg-config environment variables were complex and error-prone to configure
- Snapcraft's rust plugin is designed to handle the entire build, including cross-compilation
- The manual approach duplicated work that snapcraft would do anyway

## Solution Implemented

### Simplified Workflow Approach
The fix completely eliminates manual Rust compilation and lets snapcraft handle everything:

**Before (181 lines):**
- Manual system package installation
- Manual Rust toolchain setup
- Complex pkg-config environment configuration
- Manual cargo build
- Then snapcraft build

**After (50 lines):**
- Just uses `snapcore/action-build` with `--build-for` flag
- Snapcraft handles all compilation and cross-compilation internally

### Files Modified

#### 1. `.github/workflows/snap-ci.yml`
```yaml
jobs:
  build:
    name: Build snap (${{ matrix.platform }})
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        platform: [amd64, arm64]
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Build snap for ${{ matrix.platform }}
        uses: snapcore/action-build@v1
        id: build
        with:
          snapcraft-args: --build-for=${{ matrix.platform }}
      
      - name: Upload snap artifact
        uses: actions/upload-artifact@v4
        with:
          name: snap-${{ matrix.platform }}
          path: ${{ steps.build.outputs.snap }}
```

#### 2. `snapcraft.yaml` (in snap/ directory)
Updated platforms section to support cross-compilation:
```yaml
platforms:
  amd64:
    build-on: [amd64]
    build-for: [amd64]
  arm64:
    build-on: [amd64, arm64]    # Can build arm64 on amd64
    build-for: [arm64]
```

Added conditional build packages for cross-compilation:
```yaml
build-packages:
  - libadwaita-1-dev
  - libgtk-4-dev
  - libssl-dev
  - pkg-config
  - on amd64 to arm64:          # Only when cross-compiling
    - gcc-aarch64-linux-gnu
    - libadwaita-1-dev         # No :arm64 suffix - snapcraft handles architecture
    - libgtk-4-dev
    - libssl-dev
```

**Note:** Architecture suffixes (`:arm64`) are not needed in the conditional block because snapcraft automatically fetches packages for the target architecture when cross-compiling.

## How Cross-Compilation Works Now

1. **GitHub Actions Runner**: Runs on ubuntu-latest (amd64 architecture)

2. **Matrix Build**: Creates two parallel jobs - one for amd64, one for arm64

3. **For amd64 build**:
   - LXD container starts on amd64
   - Snapcraft rust plugin compiles natively
   - No cross-compilation needed

4. **For arm64 build**:
   - LXD container starts on amd64 host
   - Snapcraft sees platform configuration allows "build-on: [amd64]" for arm64
   - Conditional packages install cross-compilation toolchain
   - Snapcraft rust plugin sets up cross-compilation environment automatically
   - Compiles for arm64 target

5. **Build Process Inside Snapcraft**:
   - Snapcraft reads snapcraft.yaml
   - Installs build-packages (including cross-compilation tools if needed)
   - Rust plugin detects target architecture
   - Sets up appropriate Rust target and linker
   - Configures pkg-config for cross-compilation
   - Runs cargo build with correct target
   - Packages the binary into snap

## Testing the Fix

### Method 1: GitHub Web Interface (Recommended)
1. Navigate to https://github.com/makoni/actioneer-gtk/actions
2. Click on "Snap CI" in the workflows list
3. Click the "Run workflow" dropdown button
4. Select branch: `copilot/fix-snap-build-errors`
5. Click green "Run workflow" button
6. Monitor the progress of both amd64 and arm64 builds
7. Once complete, verify both artifacts are uploaded successfully

### Method 2: gh CLI
```bash
gh workflow run snap-ci.yml --ref copilot/fix-snap-build-errors
gh run watch
```

### Method 3: Local Testing (if you have snapcraft)
```bash
# Test amd64 build locally
snapcraft --build-for=amd64

# Test arm64 cross-compilation (requires qemu or proper setup)
snapcraft --build-for=arm64
```

## Expected Results

### Success Criteria
- [x] Workflow starts without errors
- [ ] amd64 build completes successfully (~10-15 minutes)
- [ ] arm64 build completes successfully (~15-20 minutes)
- [ ] Both snap files are created
- [ ] Artifacts are uploaded with names:
  - `snap-amd64` containing `actioneer_1.0.0_amd64.snap`
  - `snap-arm64` containing `actioneer_1.0.0_arm64.snap`
- [ ] No pkg-config errors appear in logs
- [ ] No glib-sys compilation errors

### If Successful
The workflow will:
1. Build snaps for both architectures
2. Upload them as GitHub Actions artifacts  
3. Be ready for publication to Snap Store (when uncommented)

## Additional Notes

### Why This Approach is Better
1. **Simpler**: 130+ fewer lines of complex configuration
2. **More Reliable**: Uses snapcraft's tested cross-compilation logic
3. **Maintainable**: No manual environment variable management
4. **Standard**: Follows snapcraft best practices
5. **Portable**: Works same way locally and in CI

### Potential Issues and Solutions

**Issue**: LXD/container fails to start
- **Solution**: This is handled by snapcore/action-build automatically

**Issue**: Cross-compilation still fails
- **Solution**: Check that conditional packages are installed by reviewing build logs

**Issue**: arm64 build takes very long
- **Expected**: Cross-compilation is slower than native builds (15-20 min vs 10-15 min)

**Issue**: Missing architecture-specific files
- **Solution**: The prime section in snapcraft.yaml excludes unused files for both architectures

## Related Documentation
- Snapcraft platforms: https://snapcraft.io/docs/snapcraft-platforms
- Snapcraft cross-compilation: https://snapcraft.io/docs/build-options#heading--cross-compiling
- GitHub Actions snapcore/action-build: https://github.com/snapcore/action-build

## Verification Checklist
- [x] YAML syntax is valid (both files)
- [x] Workflow file is simplified and clean
- [x] Platform configuration supports cross-compilation
- [x] Cross-compilation dependencies are added
- [x] Matrix strategy configured correctly
- [x] Artifacts will be uploaded with unique names
- [ ] Workflow runs successfully (pending manual trigger)
- [ ] Both architectures build without errors (pending test)
- [ ] Snap files are valid and installable (pending test)
