fun RepositoryHandler.enableMirror() {{
    all {{
        if (this is MavenArtifactRepository) {{
            val originalUrl = this.url.toString().removeSuffix("/")
            urlMappings[originalUrl]?.let {{
                logger.lifecycle("Repository[$url] is mirrored to $it")
                this.setUrl(it)
            }}
        }}
    }}
}}

val urlMappings = mapOf(
    "https://repo.maven.apache.org/maven2" to "{}",
    "https://dl.google.com/dl/android/maven2" to "{}",
    "https://plugins.gradle.org/m2" to "{}"
)

gradle.allprojects {{
    buildscript {{
        repositories.enableMirror()
    }}
    repositories.enableMirror()
}}

gradle.beforeSettings {{
    pluginManagement.repositories.enableMirror()
    dependencyResolutionManagement.repositories.enableMirror()
}}
