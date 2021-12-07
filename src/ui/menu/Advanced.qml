import QtQuick 2.15
import Qt.labs.settings 1.0

import "../components/"

MenuItem {
    text: qsTr("Advanced");
    icon: "settings";
    opened: false;

    // Settings {
    //     property alias timelineChart: timelineChart.currentIndex;
    //     property alias previewResolution: previewResolution.currentIndex;
    //     property alias renderBackground: renderBackground.text;
    //     property alias theme: themeList.currentIndex;
    // }

    Label {
        position: Label.Left;
        text: qsTr("Preview resolution");

        ComboBox {
            id: previewResolution;
            model: [qsTr("Full"), "1080p", "720p", "480p"];
            font.pixelSize: 12 * dpiScale;
            width: parent.width;
            currentIndex: 2;
            onCurrentIndexChanged: {
                let target_height = -1; // Full
                switch (currentIndex) {
                    case 1: target_height = 1080; break;
                    case 2: target_height = 720; break;
                    case 3: target_height = 480; break;
                }

                controller.set_preview_resolution(target_height, window.videoArea.vid);
            }
        }
    }
    Label {
        position: Label.Left;
        text: qsTr("Timeline chart");

        ComboBox {
            id: timelineChart;
            model: [qsTr("Gyroscope"), qsTr("Accelerometer"), qsTr("Quaternions")]; // TODO qsTr("Magnetometer")
            font.pixelSize: 12 * dpiScale;
            width: parent.width;
            onCurrentIndexChanged: {
                const chart = window.videoArea.timeline.getChart();
                chart.viewMode = currentIndex;
                controller.update_chart(chart);
            }
        }
    }
    CheckBox {
        visible: timelineChart.currentIndex == 2;
        text: qsTr("Show smoothed quaternions");
        onCheckedChanged: {
            const chart = window.videoArea.timeline.getChart();
            chart.setAxisVisible(4, checked);
            chart.setAxisVisible(5, checked);
            chart.setAxisVisible(6, checked);
            chart.setAxisVisible(7, checked);
        }
    }

    Label {
        position: Label.Left;
        text: qsTr("Render background");

        TextField {
            id: renderBackground;
            text: "#111111";
            width: parent.width;
            onTextChanged: {
                controller.set_background_color(text, window.videoArea.vid);
            }
        }
    }
    Label {
        position: Label.Left;
        text: qsTr("Theme");

        ComboBox {
            id: themeList;
            model: [qsTr("Light"), qsTr("Dark")];
            font.pixelSize: 12 * dpiScale;
            width: parent.width;
            currentIndex: 1;
            onCurrentIndexChanged: {
                const themes = ["light", "dark"];
                theme.set_theme(themes[currentIndex]);
            }
        }
    }
}
