Fonctionnalités supportées
 1. init

Initialise un dépôt Git minimal :

cargo run -- init


Crée :

.git/
 ├── objects/
 ├── refs/
 └── HEAD

 2. hash-object

Calcule le SHA-1 d’un fichier et peut l’écrire dans .git/objects.

Calcul sans écrire :

cargo run -- hash-object fichier.txt


Écrire l’objet blob dans .git/objects :

cargo run -- hash-object -w fichier.txt

 3. cat-file -p

Décompresse et affiche un objet Git (comme Git le ferait).

cargo run -- cat-file -p <sha1>

 4. write-tree

Reconstruit un objet tree représentant tout le répertoire courant.

cargo run -- write-tree


Cette commande :

parcourt les sous-dossiers récursivement

crée 1 blob par fichier

crée 1 tree par dossier

retourne le SHA-1 du tree racine

 5. ls-tree --name-only

Affiche les noms des fichiers contenus dans un tree.

cargo run -- ls-tree --name-only <sha_tree>

 6. commit-tree

Crée un objet commit Git à partir d’un tree SHA.

cargo run -- commit-tree <tree_sha> -p <parent_sha> -m "message"


Exemple pour un premier commit :

cargo run -- commit-tree <tree_sha> -p 0000000000000000000000000000000000000000 -m "Initial commit"



Les principales fonctions :

Fonction                  	Rôle
write_object	             Écrit un blob/tree/commit compressé
read_object_raw          	Décompresse un objet Git
write_blob_from_file     	Construit un blob
write_tree_rec	           Construit un tree récursivement
write_commit	             Construit un commit
hex_to_bin	               Convertit SHA hex → 20 bytes
split_header_body	        Sépare l’en-tête Git du contenu


Retourne le SHA du commit, écrit dans .git/objects.
