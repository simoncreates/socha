#[cfg(test)]
pub mod tests {
    use socha::{
        incoming::{
            ReceivedAggregation, ReceivedComMessage, ReceivedData, ReceivedFragment,
            ReceivedRelevantForRanking,
        },
        internal::{RoomMessage, Row},
        neutral::{PiranhaField, Size, Team},
    };
    use strong_xml::XmlRead;

    #[test]
    fn test_parse_fragment() {
        let xml = r#"
            <fragment name="Siegpunkte">
            <aggregation>SUM</aggregation>
            <relevantForRanking>true</relevantForRanking>
            </fragment>
        "#;

        let data: ReceivedFragment = ReceivedFragment::from_str(xml).unwrap();
        println!("{:?}", data);
        assert_eq!(
            data,
            ReceivedFragment {
                frag_name: Some("Siegpunkte".to_string()),
                aggregation: Some(ReceivedAggregation {
                    agr_content: "SUM".to_string()
                }),
                relevant_for_ranking: Some(ReceivedRelevantForRanking {
                    rfr_content: "true".to_string()
                })
            }
        );
    }
    #[test]
    fn test_parse_scores() {
        let xml = r#"
        <data class="result">
        <definition>
            <fragment name="Siegpunkte">
            <aggregation>SUM</aggregation>
            <relevantForRanking>true</relevantForRanking>
            </fragment>
            <fragment name="Schwarmgröße">
            <aggregation>AVERAGE</aggregation>
            <relevantForRanking>true</relevantForRanking>
            </fragment>
        </definition>
        <scores>
            <entry>
            <player team="ONE"/>
            <score>
                <part>0</part>
                <part>0</part>
            </score>
            </entry>
            <entry>
            <player team="TWO"/>
            <score>
                <part>2</part>
                <part>17</part>
            </score>
            </entry>
        </scores>
        <winner team="TWO" regular="false" reason="ONE hat innerhalb von 2 Sekunden nach Aufforderung keinen Zug gesendet."/>
        </data>
        "#;

        let _: ReceivedData = ReceivedData::from_str(xml).unwrap();
    }
    #[test]
    fn test_parse_game_state() {
        let xml = r#"
         <comMessage>
  <room roomId="b5f43e86-df4e-4221-b83d-337497950ac1">
    <data class="memento">
      <state class="state" startTeam="ONE" turn="0">
        <board>
          <row>
            <field>EMPTY</field>
            <field>TWO_M</field>
            <field>TWO_S</field>
            <field>TWO_L</field>
            <field>TWO_S</field>
            <field>TWO_L</field>
            <field>TWO_L</field>
            <field>TWO_M</field>
            <field>TWO_S</field>
            <field>EMPTY</field>
          </row>
          <row>
            <field>ONE_M</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>ONE_S</field>
          </row>
          <row>
            <field>ONE_S</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>SQUID</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>ONE_S</field>
          </row>
          <row>
            <field>ONE_L</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>ONE_L</field>
          </row>
          <row>
            <field>ONE_S</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>ONE_M</field>
          </row>
          <row>
            <field>ONE_L</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>ONE_M</field>
          </row>
          <row>
            <field>ONE_L</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>SQUID</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>ONE_S</field>
          </row>
          <row>
            <field>ONE_M</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>ONE_S</field>
          </row>
          <row>
            <field>ONE_S</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>EMPTY</field>
            <field>ONE_L</field>
          </row>
          <row>
            <field>EMPTY</field>
            <field>TWO_S</field>
            <field>TWO_S</field>
            <field>TWO_L</field>
            <field>TWO_M</field>
            <field>TWO_M</field>
            <field>TWO_S</field>
            <field>TWO_S</field>
            <field>TWO_L</field>
            <field>EMPTY</field>
          </row>
        </board>
      </state>
    </data>
  </room></comMessage>
        "#;

        let received_com_message = ReceivedComMessage::from_str(xml).unwrap();
        println!("{:?}", received_com_message);
        for room in received_com_message.room {
            let room = RoomMessage::try_from(room).unwrap();
            match room {
                RoomMessage::Memento(state) => {
                    assert_eq!(state.turn, 0);
                    assert_eq!(state.class, Some("state".to_string()));
                    assert_eq!(state.start_team, Team::One);

                    assert_eq!(state.board.rows.len(), 10);
                    assert_eq!(state.board.rows[0].fields.len(), 10);
                    assert_eq!(
                        state.board.rows[0],
                        Row {
                            fields: [
                                PiranhaField::Empty,
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::M
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::S
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::L
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::S
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::L
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::L
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::M
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::S
                                },
                                PiranhaField::Empty
                            ]
                        },
                    );
                    assert_eq!(
                        state.board.rows[1],
                        Row {
                            fields: [
                                PiranhaField::Fish {
                                    team: Team::One,
                                    size: Size::M
                                },
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Fish {
                                    team: Team::One,
                                    size: Size::S
                                },
                            ]
                        },
                    );
                    assert_eq!(
                        state.board.rows[6],
                        Row {
                            fields: [
                                PiranhaField::Fish {
                                    team: Team::One,
                                    size: Size::L
                                },
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Squid,
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Empty,
                                PiranhaField::Fish {
                                    team: Team::One,
                                    size: Size::S
                                },
                            ]
                        },
                    );

                    assert_eq!(
                        state.board.rows[9],
                        Row {
                            fields: [
                                PiranhaField::Empty,
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::S
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::S
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::L
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::M
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::M
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::S
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::S
                                },
                                PiranhaField::Fish {
                                    team: Team::Two,
                                    size: Size::L
                                },
                                PiranhaField::Empty,
                            ]
                        },
                    );

                    println!("turn: {}", state.turn);
                    println!("class: {:?}", state.class);
                    println!("start_team: {:?}", state.start_team);
                }
                _ => panic!("expected memento"),
            }
        }
    }
}
