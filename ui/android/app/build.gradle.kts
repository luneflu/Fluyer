plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "org.alvindimas05.fluyer"
    compileSdk = 35

    defaultConfig {
        applicationId = "org.alvindimas05.fluyer"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
        // ponytail: matches scripts/build_android.sh; add ABIs in both places.
        ndk { abiFilters += "arm64-v8a" }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
    buildFeatures { compose = true }
    testOptions { unitTests.isReturnDefaultValues = true }

    // Generated UniFFI bindings live outside the app, like bindings/swift.
    sourceSets["main"].java.srcDir("../../../bindings/kotlin")

    // Extract .so files to nativeLibraryDir so BASS_PluginLoad("libbassflac.so")
    // resolves by bare name the same way desktop loads plugins.
    packaging { jniLibs { useLegacyPackaging = true } }
}

dependencies {
    // UniFFI runtime
    implementation("net.java.dev.jna:jna:5.17.0@aar")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.10.2")

    implementation(platform("androidx.compose:compose-bom:2025.05.01"))
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-extended")
    implementation("androidx.activity:activity-compose:1.10.1")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.9.0")
    implementation("androidx.media:media:1.7.0")

    testImplementation("junit:junit:4.13.2")
    // android.jar's org.json is a stub on the JVM.
    testImplementation("org.json:json:20240303")
}
