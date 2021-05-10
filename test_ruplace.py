from ruplace import ruplace


def test_ruplace_substring():
    input = "le thé = thé !"
    pattern = "thé"
    replacement = "café"
    output = ruplace("substring", input, pattern, replacement)
    assert output == "le café = café !"


def test_ruplace_regex():
    input = "The record says 'LastName, FirstName'"
    pattern = r"(\w+), (\w+)"
    replacement = r"\2 \1"
    output = ruplace("regex", input, pattern, replacement)
    assert output == "The record says 'FirstName LastName'"


def test_ruplace_subvert():
    input = "let foo_bar = FooBar()"
    pattern = "foo_bar"
    replacement = "spam_eggs"
    output = ruplace("subvert", input, pattern, replacement)
    assert output == "let spam_eggs = SpamEggs()"
